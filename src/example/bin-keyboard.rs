extern crate hid_api_rs;

use std::{io, process, thread};
use std::io::BufWriter;

use hid_api_rs::{hid, HidSpecification};
use hid_api_rs::gadgets::keyboard;
use hid_api_rs::gadgets::keyboard::UsbKeyCode;

pub fn main() {
    let specification: HidSpecification = HidSpecification {
        mouse_inputs: None,
        keyboard_inputs: Some(
            vec![
                // TODO: User!, replace these path's to your own keyboard's! They can be found in "/dev/input/by-id/usb-X-event-kbd"
                String::from("/dev/input/by-id/usb-Keychron_K4_Keychron_K4-event-kbd")
            ]
        ),
        gadget_output: String::from("/dev/hidg0"),
    };
    
    if let Err(err) = hid_api_rs::start_pass_through(specification) {
        panic!("Failed to start pass through! {}", err)
    };
    
    // This can be either ``/dev/hidg0`` or ``/dev/hidg1``.
    // ``/dev/hidg0`` can be used but will downgrade device pass through performance
    let gadget_file = match hid::open_gadget_device(String::from("/dev/hidg1")) {
        Ok(gadget_device) => gadget_device,
        Err(_) => {
            println!("Failed to open gadget device");
            return
        }
    };
    
    let mut gadget_writer = BufWriter::new(gadget_file);
    
    static mut BREAK_LOCAL_THREAD: bool = false;
    thread::spawn(move || {
        loop {
            unsafe {
                if BREAK_LOCAL_THREAD {
                    return;
                }
            }
    
            let keyboard_state = hid_api_rs::get_keyboard();
    
            if keyboard::is_key_down(UsbKeyCode::KEYH, &keyboard_state) {
                keyboard::add_generic_down(UsbKeyCode::KEYI as i32, &keyboard_state.keys_down).unwrap();
                keyboard::remove_generic_down(UsbKeyCode::KEYH as i32, &mut keyboard_state.keys_down).unwrap();
                hid::write_keyboard(&keyboard_state, &mut gadget_writer).unwrap();
    
                keyboard::remove_generic_down(UsbKeyCode::KEYI as i32, &mut keyboard_state.keys_down).unwrap();
                hid::write_keyboard(&keyboard_state, &mut gadget_writer).unwrap();
    
                // Says "hi"
            }
        }
    });
    
    loop {
        let mut answer = String::new();
    
        io::stdin()
            .read_line(&mut answer)
            .expect("Failed to read line");
    
        if !answer.is_empty() {
            println!("Stopping");
    
            unsafe { BREAK_LOCAL_THREAD = true; }
            if let Err(error) = hid_api_rs::stop_pass_through() {
                println!("Error occurred while stopping pass through: {error}");
                process::abort();
            };
    
            break;
        }
    }
}