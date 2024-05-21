extern crate hid_api_rs;

use std::{io, process, thread};
use std::io::BufWriter;
use std::sync::Mutex;
use std::time::Duration;

use hid_api_rs::{gadgets::{
    keyboard::{self, UsbKeyCode, KeyCodeModifier},
    mouse::{self, MouseRaw},
}, hid, HidMouse, HidSpecification};

pub fn main() {
    let specification: HidSpecification = HidSpecification {
        // TODO: /dev/input/mice will usually be the same for everyone. if not find your device by-id
        mouse_inputs: Some([HidMouse { mouse_path: String::from("/dev/input/mice"), mouse_poll_rate: Some(1000) }].to_vec()),
        // TODO: USER!, replace these path's to your own keyboard paths for functionality! These will always be different for each user (Different keyboards & hardware)
        keyboard_inputs: Some([String::from("/dev/input/by-id/usb-Keychron_K4_Keychron_K4-event-kbd"), String::from("/dev/input/by-id/usb-Logitech_G502_HERO_Gaming_Mouse_0E6D395D3333-if01-event-kbd")].to_vec()),
        // keyboard_inputs: None,
        gadget_output: String::from("/dev/hidg0"),
    };

    thread::spawn(|| {
        if let Err(err) = hid_api_rs::start_pass_through(specification) {
            panic!("Failed to start pass through! {}", err)
        };
    });

    static mut BREAK_LOCAL_THREAD: bool = false;
    thread::spawn(|| {
        let mut key_switch: bool = false;

        let gadget_file = match hid::open_gadget_device(String::from("/dev/hidg1")) {
            Ok(gadget_device) => gadget_device,
            Err(_) => {
                println!("Failed to open gadget device");
                return
            }
        };

        let gadget_writer = Mutex::new(BufWriter::new(gadget_file));

        loop {
            unsafe {
                if BREAK_LOCAL_THREAD {
                    return;
                }
            }

            let keyboard_state = hid_api_rs::get_keyboard();
            if keyboard::is_key_down(UsbKeyCode::KEYY, &keyboard_state) {
                key_switch = !key_switch;
            }

            if keyboard::is_modifier_down(KeyCodeModifier::KEYLEFTMETA, &keyboard_state) {
                println!("Meta key down!");
            }

            let mouses = hid_api_rs::get_mouses();
            // println!("Total mouses in circulation: {}", mouses.len());
            for mouse_index in 0..mouses.len() {
                let mouse = &mut mouses[mouse_index];
                let mouse_state = mouse.get_state().clone();

                let left: bool = mouse_state.left_button;
                let middle: bool = mouse_state.middle_button;

                if middle {
                    mouse.mouse_settings.invert_x = !mouse.mouse_settings.invert_x;
                    mouse.mouse_settings.invert_y = !mouse.mouse_settings.invert_y;
                    mouse.mouse_settings.invert_wheel = !mouse.mouse_settings.invert_wheel;
                }

                if left {
                    let mouse_raw = MouseRaw {
                        relative_x: 25,
                        ..Default::default()
                    };

                    if let Ok(mut gadget_writer) = gadget_writer.lock() {
                        if let Err(error) = mouse::push_mouse_event(mouse_raw, Some(mouse), &mut gadget_writer) {
                            println!("Failed to push mouse event: {error}");
                        };
                    }
                    continue;
                }
            }

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

            unsafe { BREAK_LOCAL_THREAD = true; }
            if let Err(error) = hid_api_rs::stop_pass_through() {
                println!("Error occurred while stopping pass through: {error}");
                process::abort();
            };

            break;
        }
    }
}
