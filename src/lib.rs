use core::panic;
use std::io::BufWriter;
use std::sync::RwLock;
use std::{fs::File, io, path::Path, thread};

use gadgets::mouse::{self, Mouse};

pub mod gadgets;
pub mod hid;

pub struct HidSpecification {
    pub mouse_inputs: Vec<String>,
    pub keyboard_inputs: Vec<String>,
    pub gadget_output: String,
}

use crate::mouse::MouseRaw;

static mut GADGET_WRITER: Option<BufWriter<&mut File>> = None;

static mut MOUSE_INTERFACES: Vec<Mouse> = Vec::new();
// static mut KEYBOARD_INTERFACES: Vec<Test> = Vec::new();

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
                    mouse::attempt_read(mouse);

                    if let Err(_) = mouse::attempt_flush(mouse, gadget_writer) {
                        println!("failed to flush mouse")
                    };
                }
            },
            None => panic!("No gadget writer wth?!"),
        }
    }

    Ok(())
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
                        panic!("failed to flush mouse (stop passthrough)")
                    };
                }
            }
            None => panic!("Failed to find gadget file while stopping"),
        }
    }
}

pub fn start_watcher_threads(mouse_inputs: Vec<String>, mut keyboard_inputs: Vec<String>) {
    thread::spawn(move || unsafe {
        mouse::check_mouses(mouse_inputs, &mut MOUSE_INTERFACES);
    });

    // thread::spawn(move || {
    //     unsafe {
    //         // TODO: Implement keyboard
    //         // check_keyboards(specification.mouse_inputs, specification.keyboard_inputs);
    //     }
    // });
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
