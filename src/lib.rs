use core::panic;
use std::io::BufWriter;
use std::{fs::File, io, thread};

use std::sync::{Arc, Mutex};
use std::time::Duration;

use gadgets::keyboard::{self, Keyboard, KeyboardState};
use gadgets::mouse::{self, Mouse};

pub mod gadgets;
pub mod hid;

pub struct HidSpecification {
    pub mouse_inputs: Option<Vec<HidMouse>>,
    pub keyboard_inputs: Option<Vec<String>>,
    pub gadget_output: String,
}

#[derive(Clone)]
pub struct HidMouse {
    pub mouse_path: String,
    pub mouse_poll_rate: Option<i32>,
}

use once_cell::sync::Lazy;

use crate::mouse::MouseRaw;

static mut GADGET_WRITER: Option<Arc<Mutex<BufWriter<&mut File>>>> = None;

static mut MOUSE_INTERFACES: Vec<Mouse> = Vec::new();
static mut KEYBOARD_INTERFACES: Vec<Keyboard> = Vec::new();

static mut GLOBAL_KEYBOARD_STATE: Lazy<KeyboardState> = Lazy::new(|| KeyboardState::default());

pub fn start_passthrough(specification: HidSpecification) -> Result<(), io::Error> {
    let gadget_file = match hid::open_gadget_device(specification.gadget_output) {
        Ok(gadget_device) => gadget_device,
        Err(err) => return Err(err),
    };

    start_watcher_threads(specification.mouse_inputs, specification.keyboard_inputs);

    unsafe {
        GADGET_WRITER = Some(Arc::new(Mutex::new(BufWriter::new(gadget_file))));

        match &GADGET_WRITER {
            Some(gadget_writer) => {
                let mouse_gadget_writer = Arc::clone(gadget_writer);
                thread::spawn(move || {
                    loop {
                        if MOUSE_INTERFACES.is_empty() {
                            thread::sleep(Duration::from_millis(1));
                            continue;
                        }

                        for mouse_interface_index in 0..MOUSE_INTERFACES.len() {
                            let mut mouse: &mut Mouse = &mut MOUSE_INTERFACES[mouse_interface_index];
                            thread::scope(|scope| {
                                let scoped_mouse = &mut mouse;
                                scope.spawn(move || {
                                    if let Err(_) = mouse::attempt_read(*scoped_mouse) {
                                        // println!("Failed to reach mouse, ({}). Removing from interface list!", err);
                                        MOUSE_INTERFACES.remove(mouse_interface_index);
                                    };
                                });
                            });

                            if let Ok(mut writer) = mouse_gadget_writer.lock() {
                                let _ = mouse::attempt_flush(&mut mouse, &mut writer);
                            }
                        }
                    }
                });

                let keyboard_gadget_writer = Arc::clone(gadget_writer);

                thread::spawn(move || {
                    loop {
                        if KEYBOARD_INTERFACES.is_empty() {
                            thread::sleep(Duration::from_millis(1));
                            continue;
                        }

                        for keyboard_interface_index in 0..KEYBOARD_INTERFACES.len() {
                            let mut keyboard: &mut Keyboard =
                                &mut KEYBOARD_INTERFACES[keyboard_interface_index];

                            thread::scope(|scope| {
                                let scoped_keyboard = &mut keyboard;
                                scope.spawn(move || {
                                    if let Err(_) = keyboard::attempt_read(*scoped_keyboard, &mut GLOBAL_KEYBOARD_STATE) {
                                        // println!("Failed to read keyboard, ({}). Removing from interface list!", err);
                                        KEYBOARD_INTERFACES.remove(keyboard_interface_index);
                                    };
                                });
                            });
                        }

                        if let Ok(mut writer) = keyboard_gadget_writer.lock() {
                            let _ = keyboard::attempt_flush(&mut GLOBAL_KEYBOARD_STATE, &mut writer);
                        }
                    }
                });

                return Ok(());
            }
            None => panic!("No gadget writer.")
        }
    }
}

pub fn stop_passthrough() {
    unsafe {
        match &GADGET_WRITER {
            Some(gadget_writer) => {
                if let Ok(mut writer) = gadget_writer.lock() {
                    for mouse_interface_index in 0..MOUSE_INTERFACES.len() {
                        let mouse: &mut Mouse = &mut MOUSE_INTERFACES[mouse_interface_index];
                        mouse::push_mouse_event(MouseRaw::default(), mouse);

                        if let Err(_) = mouse::attempt_flush(mouse, &mut writer) {
                            panic!("failed to flush mouse")
                        };
                    }

                    static mut DEFAULT_KEYBOARD_STATE: Lazy<KeyboardState> =
                        Lazy::new(|| KeyboardState::default());
                    if let Err(err) =
                        keyboard::attempt_flush(&mut DEFAULT_KEYBOARD_STATE, &mut writer)
                    {
                        panic!("failed to flush keyboard, ({})", err)
                    };
                }
            }
            None => ()
        }
    }
}

fn start_watcher_threads(
    mouse_inputs: Option<Vec<HidMouse>>,
    keyboard_inputs: Option<Vec<String>>,
) {
    if let Some(mouse_inputs) = mouse_inputs {
        thread::spawn(move || unsafe {
            mouse::check_mouses(mouse_inputs, &mut MOUSE_INTERFACES);
        });
    }

    if let Some(keyboard_inputs) = keyboard_inputs {
        thread::spawn(move || unsafe {
            keyboard::check_keyboards(keyboard_inputs, &mut KEYBOARD_INTERFACES);
        });
    }
}

pub fn get_mouses() -> &'static mut Vec<Mouse> {
    unsafe { return &mut MOUSE_INTERFACES; }
}

pub fn get_keyboard() -> &'static mut KeyboardState {
    unsafe { return &mut GLOBAL_KEYBOARD_STATE; }
}
