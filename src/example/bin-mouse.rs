extern crate hid_api_rs;

use std::{io, process, thread};
use std::time::Duration;

use hid_api_rs::{HidMouse, HidSpecification};

pub fn main() {
    let specification: HidSpecification = HidSpecification {
        mouse_inputs: Some(
            vec![
                // TODO: "/dev/input/mice" will usually work for raspbian, however if not use "/dev/input/by-id/usb-X-event-mouse"
                HidMouse {
                    mouse_path: String::from("/dev/input/mice"),
                    mouse_poll_rate: Some(1000),
                    mouse_side_buttons: true,
                }
            ]
        ),
        keyboard_inputs: Some(
            vec![
                // TODO: User!, replace these path's to your own keyboard's! They can be found in "/dev/input/by-id/usb-X-event-kbd"
                String::from("/dev/input/by-id/usb-Keychron_K4_Keychron_K4-event-kbd"),
                String::from("/dev/input/by-id/usb-Logitech_G502_HERO_Gaming_Mouse_0E6D395D3333-if01-event-kbd")
            ]
        ),
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

            if let Ok(movement) = movement_receiver.try_recv() {
                println!("X: {} | Y: {} | WHEEL: {}", movement.relative_x, movement.relative_y, movement.relative_wheel);
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