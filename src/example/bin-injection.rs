extern crate hid_api_rs;

use std::{io, process, thread};
use std::env;
use std::io::BufWriter;
use std::sync::Mutex;
use std::time::Duration;

use hid_api_rs::gadgets::mouse;
use hid_api_rs::gadgets::mouse::MouseRaw;
use hid_api_rs::hid;

pub fn main() {
    env::set_var("RUST_BACKTRACE", "full");

    static mut BREAK_LOCAL_THREAD: bool = false;
    thread::spawn(|| {
        // This can be either ``/dev/hidg0`` or ``/dev/hidg1``.
        // Both are free since we are only injecting not full pass through.
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

            let mouse_raw = MouseRaw {
                relative_x: 25,
                ..Default::default()
            };

            if let Ok(mut gadget_writer) = gadget_writer.lock() {
                if let Err(error) = mouse::push_mouse_event(mouse_raw, None, &mut gadget_writer) {
                    println!("Failed to push mouse event: {error}");
                };
            }

            thread::sleep(Duration::from_millis(1000))
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