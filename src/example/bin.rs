use hid_api_rs::{
    gadgets::{
        keyboard::{self, UsbKeyCode},
        mouse::{self, MouseRaw},
    },
    HidMouse, HidSpecification,
};
use std::{io, thread};

extern crate hid_api_rs;

pub fn main() {
    let specification: HidSpecification = HidSpecification {
        mouse_inputs: Some([HidMouse {mouse_path: String::from("/dev/input/mice"), mouse_poll_rate: Some(1000)}].to_vec()),
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
            if keyboard::is_key_down(UsbKeyCode::KEYY, &keyboard_state) {
                key_switch = !key_switch;
            }

            if keyboard::is_modifier_down(keyboard::KeyCodeModifier::KEYLEFTMETA, &keyboard_state) {
                println!("Meta key down!");
            }

            let mouses = hid_api_rs::get_mouses();
            // println!("Total mouses in circulation: {}", mouses.len());
            for mouse_index in 0..mouses.len() {
                let mouse = &mut mouses[mouse_index];

                let mut left: bool = false;
                if let Ok(left_button) = &mouse.mouse_state.left_button.try_read() {
                    left = **left_button;
                    drop(left_button);

                    // println!("left: {}", left);
                    // println!("Key Switch: {}", key_switch);
                };

                if left {
                    let mouse_raw = MouseRaw {
                        left_button: Some(false),
                        relative_x: 25,
                        ..Default::default()
                    };

                    mouse::push_mouse_event_low_priority(mouse_raw, mouse);
                    continue;
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

                            if let Ok(mut invert_wheel) = mouse.mouse_state.invert_wheel.try_write()
                            {
                                *invert_wheel = true;
                            }
                        } else {
                            if let Ok(mut invert_x) = mouse.mouse_state.invert_x.try_write() {
                                *invert_x = false;
                            }

                            if let Ok(mut invert_y) = mouse.mouse_state.invert_y.try_write() {
                                *invert_y = false;
                            }

                            if let Ok(mut invert_wheel) = mouse.mouse_state.invert_wheel.try_write()
                            {
                                *invert_wheel = false;
                            }
                        }
                        switcher = !switcher;
                    }
                }
                // println!("Left: {}, Right: {}, Middle: {}", mouse.mouse_state.left_button.read().unwrap(), mouse.mouse_state.right_button.read().unwrap(), mouse.mouse_state.middle_button.read().unwrap());
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
