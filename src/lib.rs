use core::panic;
use std::io::BufWriter;
use std::{fs::File, io, thread};

use gadgets::mouse::{self, Mouse};
use gadgets::keyboard::{self, Keyboard, KeyboardState};

pub mod gadgets;
pub mod hid;

pub struct HidSpecification {
    pub mouse_inputs: Option<Vec<String>>,
    pub keyboard_inputs: Option<Vec<String>>,
    pub gadget_output: String,
}

use once_cell::sync::Lazy;

use crate::mouse::MouseRaw;

static mut GADGET_WRITER: Option<BufWriter<&mut File>> = None;

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
        GADGET_WRITER = Some(BufWriter::new(gadget_file));
        match &mut GADGET_WRITER {
            Some(gadget_writer) => loop {
                for mouse_interface_index in 0..MOUSE_INTERFACES.len() {
                    let mouse: &mut Mouse = &mut MOUSE_INTERFACES[mouse_interface_index];
                    if let Err(err) = mouse::attempt_read(mouse) {
                        println!("failed to reach mouse, ({})", err)
                    };

                    if let Err(err) = mouse::attempt_flush(mouse, gadget_writer) {
                        panic!("failed to flush mouse, ({})", err)
                    };
                }

                for keyboard_interface_index in 0..KEYBOARD_INTERFACES.len() {
                    let keyboard: &mut Keyboard = &mut KEYBOARD_INTERFACES[keyboard_interface_index];
                    if let Err(err) = keyboard::attempt_read(keyboard) {
                        println!("failed to reach mouse, ({})", err)
                    };
                }

                
            },
            None => panic!("No gadget writer wth?!"),
        }
    }
}

pub fn stop_passthrough() {
    // TODO: Clear all Mouse and Keyboard buffers to zero
    unsafe {
        match &mut GADGET_WRITER {
            Some(gadget_writer) => {
                for mouse_interface_index in 0..MOUSE_INTERFACES.len() {
                    let mouse: &mut Mouse = &mut MOUSE_INTERFACES[mouse_interface_index];
                    mouse::push_mouse_event(MouseRaw::default(), mouse);

                    if let Err(_) = mouse::attempt_flush(mouse, gadget_writer) {
                        panic!("failed to flush mouse")
                    };
                }
            }
            None => panic!("Failed to find gadget file while stopping"),
        }
    }
}

pub fn start_watcher_threads(mouse_inputs: Option<Vec<String>>, mut _keyboard_inputs: Option<Vec<String>>) {
    if let Some(mouse_inputs) = mouse_inputs {
        thread::spawn(move || unsafe {
            mouse::check_mouses(mouse_inputs, &mut MOUSE_INTERFACES);
        });
    }

    // thread::spawn(move || {
    //     unsafe {
    //         // TODO: Implement keyboard
    //         // check_keyboards(specification.mouse_inputs, specification.keyboard_inputs);
    //     }
    // });
}

pub fn get_mouses() -> &'static mut Vec<Mouse> {
    unsafe {
        return &mut MOUSE_INTERFACES
    }
}

pub fn get_keyboards() -> &'static mut Vec<Keyboard> {
    unsafe {
        return &mut KEYBOARD_INTERFACES
    }
}