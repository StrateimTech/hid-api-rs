use std::{io, thread};

use hid_api_rs::{
    gadgets::{
        keyboard::{self, UsbKeyCode}
    },
    HidSpecification,
};

extern crate hid_api_rs;

pub fn main() {
    let specification: HidSpecification = HidSpecification {
        mouse_inputs: Some([String::from("/dev/input/mice")].to_vec()),
        keyboard_inputs: Some([String::from("/dev/input/by-id/usb-Keychron_K4_Keychron_K4-event-kbd"), String::from("/dev/input/by-id/usb-Logitech_G502_HERO_Gaming_Mouse_0E6D395D3333-if01-event-kbd")].to_vec()),
        gadget_output: String::from("/dev/hidg0"),
    };

    thread::spawn(|| {
        if let Err(err) = hid_api_rs::start_passthrough(specification) {
            panic!("Failed to start passthrough! {}", err)
        };
    });

    thread::spawn(|| {
        let mut switcher: bool = false;
        let mut key_switch: bool = false;

        loop {
            let keyboard_state = hid_api_rs::get_keyboard();

            // if let Ok(ref modifier_rwl) = keyboard_state.modifier.try_read() {
            //     if let Some(modifier) = modifier_rwl.deref() {
            //         println!("Modifier: {}", modifier.to_string());
            //     }
            // }

            // if let Ok(ref keys_down) = keyboard_state.keys_down.try_read() {
            //     for key_down in keys_down.to_vec() {
            //         if let Ok(key) = UsbKeyCode::try_from(key_down as i16) {
            //             println!("Key Down: {}", key.to_string())
            //         };
            //     }
            // }

            if keyboard::is_key_down(UsbKeyCode::KEYY, keyboard_state) {
                key_switch = !key_switch;
            }

            // println!("Total mouses in circulation: {}",h id_api_rs::get_mouses().len());
            for mouse in hid_api_rs::get_mouses() {
                if let Ok(left_button) = mouse.mouse_state.left_button.try_read() {
                    if *left_button == true {
                        drop(left_button);
                        
                        println!("left: True");
                        println!("Key Switch: {}", key_switch)
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
