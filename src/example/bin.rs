use std::thread;

use hid_api_rs::{gadgets::mouse, HidSpecification};

extern crate hid_api_rs;

pub fn main() {
    let specification: HidSpecification = HidSpecification {
        mouse_inputs: Some([String::from("/dev/input/mice")].to_vec()),
        keyboard_inputs: None,
        gadget_output: String::from("/dev/hidg0"),
    };

    thread::spawn(|| {
        if let Err(err) = hid_api_rs::start_passthrough(specification) {
            panic!("Failed to start passthrough! {}", err)
        };
    });

    let mut switcher: bool = false;

    loop {
        // println!("Total mouses in circulation: {}",h id_api_rs::get_mouses().len());
        for mouse in hid_api_rs::get_mouses() {
            if let Ok(left_button) = mouse.mouse_state.left_button.try_read() {
                if *left_button == true {
                    drop(left_button);
                    println!("left: True");
                }
            }

            if let Ok(middle_button) = mouse.mouse_state.middle_button.try_read() {
                if *middle_button == true {
                    println!("Inverting!");
                    if switcher == false {
                        if let Ok(mut invert_x) = mouse.mouse_state.invert_x.try_write() {
                            *invert_x = true;
                        }

                        if let Ok(mut invert_y) = mouse.mouse_state.invert_y.try_write() {
                            *invert_y = true;
                        }

                        if let Ok(mut invert_wheel) = mouse.mouse_state.invert_wheel.try_write() {
                            *invert_wheel = true;
                        }
                    } else {
                        if let Ok(mut invert_x) = mouse.mouse_state.invert_x.try_write() {
                            *invert_x = false;
                        }

                        if let Ok(mut invert_y) = mouse.mouse_state.invert_y.try_write() {
                            *invert_y = false;
                        }

                        if let Ok(mut invert_wheel) = mouse.mouse_state.invert_wheel.try_write() {
                            *invert_wheel = false;
                        }
                    }
                    switcher = !switcher;
                }
            }
            // println!("Left: {}, Right: {}, Middle: {}", mouses.mouse_state.left_button.read()., mouses.mouse_state.right_button.read().unwrap(), mouses.mouse_state.middle_button.read().unwrap());
        }
    }
}
