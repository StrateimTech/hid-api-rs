extern crate hid_api_rs;

use std::{io, process, thread};
use std::env;
use std::time::Duration;

use hid_api_rs::{HidMouse, HidSpecification};

pub fn main() {
    env::set_var("RUST_BACKTRACE", "full");

    let specification: HidSpecification = HidSpecification {
        // TODO: /dev/input/mice will usually be the same for everyone. if not find your device by-id
        mouse_inputs: Some([HidMouse { mouse_path: String::from("/dev/input/mice"), mouse_poll_rate: Some(1000), mouse_side_buttons: true }].to_vec()),
        // TODO: USER!, replace these path's to your own keyboard paths for functionality! These will always be different for each user (Different keyboards & hardware)
        keyboard_inputs: Some([String::from("/dev/input/by-id/usb-Keychron_K4_Keychron_K4-event-kbd"), String::from("/dev/input/by-id/usb-Logitech_G502_HERO_Gaming_Mouse_0E6D395D3333-if01-event-kbd")].to_vec()),
        // keyboard_inputs: None,
        gadget_output: String::from("/dev/hidg0"),
    };

    if let Err(err) = hid_api_rs::start_pass_through(specification) {
        panic!("Failed to start pass through! {}", err)
    };

    static mut BREAK_LOCAL_THREAD: bool = false;
    thread::spawn(|| {
        loop {
            unsafe {
                if BREAK_LOCAL_THREAD {
                    return;
                }
            }

            let mouses = hid_api_rs::get_mouses();

            for mouse in mouses.iter_mut() {
                let mouse_state = *mouse.get_state();

                if mouse_state.middle_button {
                    mouse.mouse_settings.invert_y = !mouse.mouse_settings.invert_y;
                    println!("Inverted Y: {}", mouse.mouse_settings.invert_y);
                }

                println!("Left: {}, Right: {}, Middle: {}, Side-4: {}, Side-5: {}", mouse_state.left_button, mouse_state.right_button, mouse_state.middle_button, mouse_state.four_button, mouse_state.five_button);
            }

            thread::sleep(Duration::from_millis(500))
        }
    });

    thread::spawn(|| loop {
        unsafe {
            if BREAK_LOCAL_THREAD {
                return;
            }
        }

        let mouses = hid_api_rs::get_mouses();

        for mouse in mouses.iter() {
            let movement_receiver = mouse.get_movement();

            match movement_receiver.try_recv() {
                Ok(movement) => {
                    println!("X: {} | Y: {} | WHEEL: {}", movement.relative_x, movement.relative_y, movement.relative_wheel);
                },
                Err(_) => {}
            };
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