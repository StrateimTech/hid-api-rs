use hid_api_rs::{
    gadgets::{
        keyboard::{self, UsbKeyCode},
        mouse::{self, MouseRaw},
    },
    HidMouse, HidSpecification,
};
use std::{io, thread};
use std::time::Duration;

extern crate hid_api_rs;

pub fn main() {
    let specification: HidSpecification = HidSpecification {
        // TODO: /dev/input/mice will usually be the same for everyone. if not find your device by-id
        mouse_inputs: Some([HidMouse { mouse_path: String::from("/dev/input/mice"), mouse_poll_rate: Some(1000) }].to_vec()),
        // TODO: USER!, replace these path's to your own keyboard paths for functionaility! These will always be different for each user (Different keyboards & hardware)
        keyboard_inputs: Some([String::from("/dev/input/by-id/usb-Keychron_K4_Keychron_K4-event-kbd"), String::from("/dev/input/by-id/usb-Logitech_G502_HERO_Gaming_Mouse_0E6D395D3333-if01-event-kbd")].to_vec()),
        // keyboard_inputs: None,
        gadget_output: String::from("/dev/hidg0"),
    };

    thread::spawn(|| {
        if let Err(err) = hid_api_rs::start_passthrough(specification) {
            panic!("Failed to start pass through! {}", err)
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
                let mut middle: bool = false;

                if let Ok(mouse_state) = mouse.mouse_state.try_read() {
                    left = mouse_state.left_button;
                    middle = mouse_state.middle_button;
                }

                if middle {
                    if let Ok(mut mouse_state) = mouse.mouse_state.try_write() {
                        if switcher == false {
                            mouse_state.invert_x = true;
                            mouse_state.invert_y = true;
                            mouse_state.invert_wheel = true;
                        } else {
                            mouse_state.invert_x = false;
                            mouse_state.invert_y = false;
                            mouse_state.invert_wheel = false;
                        }
                        switcher = !switcher;
                    }
                }

                if left {
                    let mouse_raw = MouseRaw {
                        left_button: Some(false),
                        relative_x: 25,
                        ..Default::default()
                    };

                    // Pushing low priority will prioritise the physical device input before program input for continuity.
                    mouse::push_mouse_event_low_priority(mouse_raw, mouse);
                    continue;
                }
            }

            // You should sleep as it allows the library to access mouse_state for a brief period reducing or removing device stutters.
            thread::sleep(Duration::from_millis(1));
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
