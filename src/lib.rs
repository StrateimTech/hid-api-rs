use std::io::BufWriter;
use std::{io, thread, fs::File, path::Path};
use std::sync::RwLock;

use gadgets::mouse::{Mouse, self};

pub mod gadgets;
pub mod hid;

pub struct HidSpecification {
    pub mouse_inputs: Vec<String>,
    pub keyboard_inputs: Vec<String>,
    pub gadget_output: String
}

// Maybe RwThreadLocking
static mut MOUSE_INTERFACES: RwLock<Vec<Mouse>> = RwLock::new(Vec::new());
// static mut KEYBOARD_INTERFACES: Vec<Test> = Vec::new();

pub fn start_passthrough(specification: HidSpecification) -> Result<(), io::Error> {
    let gadget_device = match hid::open_gadget_device(specification.gadget_output) {
        Ok(gadget_device) => gadget_device,
        Err(err) => return Err(err)
    };

    let mut gadget_writer = BufWriter::new(gadget_device);

    start_watcher_threads(specification.mouse_inputs, specification.keyboard_inputs); 
    
    unsafe {
        loop {
            if let Ok(mut mouse_interfaces) = MOUSE_INTERFACES.try_write() {
                for mouse_interface_index in 0..mouse_interfaces.len() {
                    let mouse = &mut mouse_interfaces[mouse_interface_index];
                    mouse::attempt_read(mouse);
                    
                    if let Err(_) = mouse::attempt_flush(mouse, &mut gadget_writer) {
                        println!("failed to flush mouse")
                    };
                }
            };
        }
    }
    Ok(())
}

pub fn stop_passthrough() {
    // TODO: Clear all Mouse and Keyboard buffers to zero
}
 
pub fn start_watcher_threads(mouse_inputs: Vec<String>, mut keyboard_inputs: Vec<String>) {
    thread::spawn(move || {
        unsafe {
            mouse::check_mouses(mouse_inputs, &mut MOUSE_INTERFACES);
        }
    });

    // thread::spawn(move || {
    //     unsafe {
    //         // TODO: Implement keyboard
    //         // check_keyboards(specification.mouse_inputs, specification.keyboard_inputs);
    //     }
    // });
}


// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
