use std::io::{BufWriter, Error, ErrorKind};
use std::thread;
use std::time::Duration;

use once_cell::sync::Lazy;

use gadgets::keyboard::{self, Keyboard, KeyboardState};
use gadgets::mouse::{self, Mouse};

use crate::gadgets::mouse::MouseRaw;

pub mod gadgets;
pub mod hid;

#[derive(Clone)]
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

static mut HID_SPEC: Option<HidSpecification> = None;
static mut MOUSE_INTERFACES: Lazy<Vec<Mouse>> = Lazy::new(|| Vec::new());
static mut MOUSE_READING: bool = true;

static mut KEYBOARD_INTERFACES: Vec<Keyboard> = Vec::new();
static mut KEYBOARD_READING: bool = true;

static mut GLOBAL_KEYBOARD_STATE: Lazy<KeyboardState> = Lazy::new(|| KeyboardState::default());

pub fn start_pass_through(specification: HidSpecification) -> Result<(), Error> {
    unsafe {
        HID_SPEC = Some(specification.clone());
    }

    let gadget_device_keyboard = match hid::open_gadget_device(specification.gadget_output.clone()) {
        Ok(gadget_device) => gadget_device,
        Err(err) => return Err(err),
    };

    start_hot_reload(specification.mouse_inputs, specification.keyboard_inputs);

    unsafe {
        thread::spawn(move || {
            static mut MOUSE_THREADS: Vec<String> = Vec::new();
            loop {
                if !MOUSE_READING {
                    return;
                }

                if MOUSE_INTERFACES.is_empty() {
                    thread::sleep(Duration::from_millis(1));
                    continue;
                }

                for mouse_interface_index in 0..MOUSE_INTERFACES.len() {
                    let mouse: &mut Mouse = &mut MOUSE_INTERFACES[mouse_interface_index];

                    if !MOUSE_THREADS.contains(&mouse.mouse_path) || MOUSE_THREADS.is_empty() {
                        let gadget_mouse = match hid::open_gadget_device(specification.gadget_output.clone()) {
                            Ok(gadget_device) => gadget_device,
                            Err(_) => continue,
                        };

                        MOUSE_THREADS.push(mouse.mouse_path.clone());

                        let mut mouse_writer = BufWriter::new(gadget_mouse);
                        thread::spawn(move || {
                            let mouse_index = mouse_interface_index.clone();
                            loop {
                                if !MOUSE_READING {
                                    break;
                                }

                                if let Err(_) = mouse::attempt_read(mouse, &mut mouse_writer) {
                                    MOUSE_INTERFACES.remove(mouse_index);
                                    MOUSE_THREADS.remove(mouse_index);

                                    break;
                                };
                            }
                        });
                    }
                }
            }
        });

        thread::spawn(move || {
            let mut keyboard_writer = BufWriter::new(gadget_device_keyboard);

            static mut KEYBOARD_THREADS: Vec<String> = Vec::new();
            loop {
                if !KEYBOARD_READING {
                    break;
                }

                if KEYBOARD_INTERFACES.is_empty() {
                    thread::sleep(Duration::from_millis(1));
                    continue;
                }

                for keyboard_interface_index in 0..KEYBOARD_INTERFACES.len() {
                    let keyboard: &mut Keyboard = &mut KEYBOARD_INTERFACES[keyboard_interface_index];

                    if !KEYBOARD_THREADS.contains(&keyboard.keyboard_path) || KEYBOARD_THREADS.is_empty() {
                        KEYBOARD_THREADS.push(keyboard.keyboard_path.clone());

                        thread::spawn(move || {
                            let keyboard_index = keyboard_interface_index.clone();

                            loop {
                                if !KEYBOARD_READING {
                                    break;
                                }

                                if let Err(err) = keyboard::attempt_read(keyboard, &mut GLOBAL_KEYBOARD_STATE) {
                                    println!("Failed to read keyboard, ({}). Removing from interface list!", err);

                                    KEYBOARD_INTERFACES.remove(keyboard_index);
                                    KEYBOARD_THREADS.remove(keyboard_index);

                                    break;
                                };
                            }
                        });
                    }
                }

                if let Err(err) = keyboard::attempt_flush(&mut GLOBAL_KEYBOARD_STATE, &mut keyboard_writer) {
                    println!("Failed to flush keyboard state: {err}")
                }
            }
        });
    }

    Ok(())
}

pub fn stop_pass_through() -> Result<(), Error> {
    unsafe {
        MOUSE_READING = false;
        KEYBOARD_READING = false;

        return match &HID_SPEC {
            Some(spec) => {
                let gadget_device = match hid::open_gadget_device(spec.gadget_output.clone()) {
                    Ok(gadget_device) => gadget_device,
                    Err(err) => return Err(err),
                };

                let mut gadget_writer = BufWriter::new(gadget_device);

                MOUSE_INTERFACES.clear();
                if let Err(err) = mouse::push_mouse_event(MouseRaw::default(), None, &mut gadget_writer) {
                    return Err(err);
                }

                KEYBOARD_INTERFACES.clear();
                static mut DEFAULT_KEYBOARD_STATE: Lazy<KeyboardState> =
                    Lazy::new(|| KeyboardState::default());
                if let Err(err) =
                    keyboard::attempt_flush(&mut DEFAULT_KEYBOARD_STATE, &mut gadget_writer)
                {
                    panic!("failed to flush keyboard, ({})", err)
                };

                Ok(())
            }
            None => Err(Error::new(
                ErrorKind::Other,
                String::from("Hid specification not defined cannot open gadget device"),
            ))
        };
    }
}

fn start_hot_reload(
    mouse_inputs: Option<Vec<HidMouse>>,
    keyboard_inputs: Option<Vec<String>>,
) {
    if let Some(mouse_inputs) = mouse_inputs {
        if !mouse_inputs.is_empty() {
            thread::spawn(move || unsafe {
                loop {
                    if !MOUSE_READING {
                        break;
                    }

                    mouse::check_mouses(&mouse_inputs, &mut MOUSE_INTERFACES);
                }
            });
        }
    }

    if let Some(keyboard_inputs) = keyboard_inputs {
        if !keyboard_inputs.is_empty() {
            thread::spawn(move || unsafe {
                loop {
                    if !KEYBOARD_READING {
                        break;
                    }

                    keyboard::check_keyboards(&keyboard_inputs, &mut KEYBOARD_INTERFACES);
                }
            });
        }
    }
}

pub fn get_mouses() -> &'static mut Lazy<Vec<Mouse>, fn() -> Vec<Mouse>> {
    unsafe { return &mut MOUSE_INTERFACES; }
}

pub fn get_keyboard() -> &'static mut KeyboardState {
    unsafe { return &mut GLOBAL_KEYBOARD_STATE; }
}
