use hid_api_rs::{
    HidMouse, HidSpecification,
};
use std::{io, thread};

extern crate hid_api_rs;

use std::env;
use std::time::Duration;

pub fn main() {
    env::set_var("RUST_BACKTRACE", "full");

    let specification: HidSpecification = HidSpecification {
        mouse_inputs: Some([HidMouse {mouse_path: String::from("/dev/input/mice"), mouse_poll_rate: Some(1000)}].to_vec()),
        keyboard_inputs: Some([String::from("/dev/input/by-id/usb-Logitech_G502_HERO_Gaming_Mouse_0E6D395D3333-if01-event-kbd")].to_vec()),
        gadget_output: String::from("/dev/hidg0"),
    };

    thread::spawn(|| {
        if let Err(err) = hid_api_rs::start_passthrough(specification) {
            panic!("Failed to start pass through! {}", err)
        };
    });

    thread::spawn(|| {
        loop {
            let mouses = hid_api_rs::get_mouses();

            for mouse_index in 0..mouses.len() {
                let mouse = &mut mouses[mouse_index];

                if let Ok(mouse_state) = mouse.mouse_state.try_read() {
                    println!("Left: {}, Right: {}, Middle: {}", mouse_state.left_button, mouse_state.right_button, mouse_state.middle_button);
                }

                // println!("Left: {}, Right: {}, Middle: {}", mouse.mouse_state.left_button.try_read().unwrap(), mouse.mouse_state.right_button.try_read().unwrap(), mouse.mouse_state.middle_button.try_read().unwrap());
            }

            thread::sleep(Duration::from_millis(500))
        }
    });

    loop {
        let mut answer = String::new();

        io::stdin()
            .read_line(&mut answer)
            .ok()
            .expect("Failed to read line");
        if !answer.is_empty() {
            println!("Stopping");
            hid_api_rs::stop_passthrough();

            break;
        }
    }
}
