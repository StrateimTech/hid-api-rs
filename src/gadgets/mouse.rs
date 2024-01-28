use std::fs::{File, OpenOptions};
use std::path::Path;

use std::io::{BufWriter, Error, ErrorKind, Read, Write};
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

use crate::{hid, HidMouse};

pub struct Mouse {
    pub mouse_data_buffer: RwLock<Vec<MouseRaw>>,
    pub mouse_state: RwLock<MouseState>,
    // Option<File> may be redundant as check_mouse() ensures File will always exist as it's a primary required component
    pub mouse_device_file: Option<File>,
    pub mouse_path: String,
}

#[derive(Copy, Clone)]
pub struct MouseState {
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
    pub four_button: bool,
    pub five_button: bool,

    pub invert_x: bool,
    pub invert_y: bool,
    pub invert_wheel: bool,

    pub sensitivity_multiplier: i16,
}

impl Default for MouseState {
    fn default() -> Self {
        MouseState {
            left_button: false,
            right_button: false,
            middle_button: false,
            four_button: false,
            five_button: false,
            invert_x: false,
            invert_y: false,
            invert_wheel: false,
            sensitivity_multiplier: 1,
        }
    }
}

pub struct MouseRaw {
    pub left_button: Option<bool>,
    pub right_button: Option<bool>,
    pub middle_button: Option<bool>,
    pub four_button: Option<bool>,
    pub five_button: Option<bool>,

    pub relative_x: i16,
    pub relative_y: i16,
    pub relative_wheel: i16,
}

impl Default for MouseRaw {
    fn default() -> Self {
        MouseRaw {
            left_button: None,
            right_button: None,
            middle_button: None,
            four_button: None,
            five_button: None,
            relative_x: 0,
            relative_y: 0,
            relative_wheel: 0,
        }
    }
}

pub fn attempt_read(mouse: &mut Mouse) -> Result<(), Error> {
    const BUFFER_LENGTH: usize = 4;

    match mouse.mouse_device_file {
        Some(ref mut mouse_file) => {
            let mut mouse_buffer = [0u8; BUFFER_LENGTH];

            let mouse_read_length = match mouse_file.read(&mut mouse_buffer) {
                Ok(result) => result,
                Err(err) => return Err(err),
            };

            if mouse_read_length >= BUFFER_LENGTH {
                let left_button = mouse_buffer[0] & 0x1 > 0;
                let right_button = mouse_buffer[0] & 0x2 > 0;
                let middle_button = mouse_buffer[0] & 0x4 > 0;

                let mut mouse_four = false;
                let mut mouse_five = false;

                // https://isdaman.com/alsos/hardware/mouse/ps2interface.htm
                if mouse_read_length == 4 {
                    mouse_four = mouse_buffer[3] & 0x10 > 0;
                    mouse_five = mouse_buffer[3] & 0x20 > 0;
                }

                let mut relative_y = (i8::from_be_bytes(mouse_buffer[2].to_be_bytes()) * -1) as i16;
                let mut relative_x = (i8::from_be_bytes(mouse_buffer[1].to_be_bytes())) as i16;

                let mut relative_wheel = (i8::from_be_bytes(mouse_buffer[3].to_be_bytes()) * -1) as i16;
                if mouse_read_length == 4 {
                    let mut z = ((mouse_buffer[3] & 0x8) | (mouse_buffer[3] & 0x4) | (mouse_buffer[3] & 0x2) | (mouse_buffer[3] & 0x1)) as i8;

                    if mouse_buffer[3] & 0x8 > 0 {
                        z = (z << 4) >> 4
                    }

                    relative_wheel = (i8::from_be_bytes(z.to_be_bytes()) * -1) as i16;
                }

                if let Ok(mouse_state) = &mouse.mouse_state.try_read() {
                    if mouse_state.invert_x {
                        relative_x *= -1;
                    }

                    if mouse_state.invert_y {
                        relative_y *= -1;
                    }

                    if mouse_state.invert_wheel {
                        relative_wheel *= -1;
                    }

                    relative_x *= mouse_state.sensitivity_multiplier;
                    relative_y *= mouse_state.sensitivity_multiplier;
                }

                let raw_mouse = MouseRaw {
                    left_button: Some(left_button),
                    right_button: Some(right_button),
                    middle_button: Some(middle_button),
                    four_button: Some(mouse_four),
                    five_button: Some(mouse_five),
                    relative_x,
                    relative_y,
                    relative_wheel,
                };

                // println!("X: {}, Y: {}, WHEEL: {}, LEFT: {}, RIGHT: {}, MIDDLE: {}", relative_x, relative_y, relative_wheel, left_button, right_button, middle_button);
                push_mouse_event(raw_mouse, mouse);
            }
            return Ok(());
        }
        None => {
            return Err(Error::new(
                ErrorKind::Other,
                String::from("Failed to find mouse device file!"),
            ));
        }
    }
}

pub fn push_mouse_event(raw_data: MouseRaw, mouse: &mut Mouse) {
    if let Ok(mut mouse_state) = mouse.mouse_state.write() {
        if let Some(raw_left_button) = raw_data.left_button {
            mouse_state.left_button = raw_left_button;
        }

        if let Some(raw_right_button) = raw_data.right_button {
            mouse_state.right_button = raw_right_button;
        }

        if let Some(raw_middle_button) = raw_data.middle_button {
            mouse_state.middle_button = raw_middle_button;
        }

        if let Some(raw_four_button) = raw_data.four_button {
            mouse_state.four_button = raw_four_button;
        }

        if let Some(raw_five_button) = raw_data.five_button {
            mouse_state.five_button = raw_five_button;
        }

        if let Ok(mut buffer) = mouse.mouse_data_buffer.write() {
            buffer.push(raw_data);
        }
    };
}

pub fn push_mouse_event_low_priority(raw_data: MouseRaw, mouse: &mut Mouse) {
    if let Ok(mut mouse_state) = mouse.mouse_state.try_write() {
        if let Some(raw_left_button) = raw_data.left_button {
            mouse_state.left_button = raw_left_button;
        }

        if let Some(raw_right_button) = raw_data.right_button {
            mouse_state.right_button = raw_right_button;
        }

        if let Some(raw_middle_button) = raw_data.middle_button {
            mouse_state.middle_button = raw_middle_button;
        }

        if let Some(raw_four_button) = raw_data.four_button {
            mouse_state.four_button = raw_four_button;
        }

        if let Some(raw_five_button) = raw_data.five_button {
            mouse_state.five_button = raw_five_button;
        }

        if let Ok(mut buffer) = mouse.mouse_data_buffer.write() {
            buffer.push(raw_data);
        }
    }
}

pub fn check_mouses(mouse_inputs: Vec<HidMouse>, mouse_interfaces: &'static mut Vec<Mouse>) {
    loop {
        for mouse_input in &mouse_inputs {
            if mouse_interfaces.iter().any(|x| mouse_inputs.iter().any(|y| y.mouse_path == x.mouse_path)) {
                thread::sleep(Duration::from_millis(1));
                continue;
            }

            let mouse_path = Path::new(&mouse_input.mouse_path);
            if Path::exists(mouse_path) {
                // println!("Found new mouse adding to pool, ({})", &mouse_path.to_string_lossy());
                let mouse = match OpenOptions::new()
                    .write(true)
                    .read(true)
                    .open(mouse_path)
                {
                    Ok(result) => result,
                    Err(_) => continue,
                };

                let mut mouse_interface = Mouse {
                    mouse_data_buffer: RwLock::new(Vec::new()),
                    mouse_state: RwLock::from(MouseState::default()),
                    mouse_device_file: Some(mouse),
                    mouse_path: mouse_input.mouse_path.clone(),
                };

                if let Err(err) = write_magic_scroll_feature(&mut mouse_interface, true) {
                    panic!("Failed to write mouse scroll feature! {}", err);
                }

                mouse_interfaces.push(mouse_interface);
            }
        }
    }
}

pub fn attempt_flush(
    mouse: &mut Mouse,
    gadget_writer: &mut BufWriter<&mut File>,
) -> Result<(), Error> {
    match mouse.mouse_data_buffer.read() {
        Ok(mouse_buffer) => {
            // println!("Attempting data buffer flush! (len: {})", mouse_buffer.len());
            for mouse_raw in mouse_buffer.iter() {
                if let Err(err) = hid::write_mouse(mouse_raw, gadget_writer) {
                    return Err(err);
                }
            }
        }
        Err(err) => {
            return Err(Error::new(
                ErrorKind::Other,
                String::from(format!(
                    "Failed to read from mouse data buffer poisoned! ({})",
                    err
                )),
            ));
        }
    }

    mouse.mouse_data_buffer = RwLock::new(Vec::new());
    Ok(())
}

// https://wiki.osdev.org/PS/2_Mouse
pub fn write_magic_scroll_feature(mouse: &mut Mouse, side_buttons: bool) -> Result<(), Error> {
    let mut mouse_scroll: [u8; 6] = [0xf3, 200, 0xf3, 100, 0xf3, 80];

    if side_buttons {
        mouse_scroll = [0xf3, 200, 0xf3, 200, 0xf3, 80];
    }

    if let Some(ref mut mouse_data_buffer) = mouse.mouse_device_file {
        match mouse_data_buffer.write_all(&mouse_scroll) {
            Ok(_) => {
                match mouse_data_buffer.flush() {
                    Ok(_) => return Ok(()),
                    Err(err) => return Err(err),
                };
            }
            Err(err) => return Err(err),
        };
    }

    Err(Error::new(
        ErrorKind::Other,
        String::from("Failed push magic scroll feature to mouse!"),
    ))
}

pub fn write_poll_rate(mouse: &mut Mouse, poll_rate: i32) -> Result<(), Error> {
    let mut poll_rate_packet: Vec<u8> = Vec::new();
    poll_rate_packet.push(0xf3);
    poll_rate_packet.append(&mut poll_rate.to_be_bytes().to_vec());

    if let Some(ref mut mouse_data_buffer) = mouse.mouse_device_file {
        match mouse_data_buffer.write_all(&poll_rate_packet) {
            Ok(_) => {
                match mouse_data_buffer.flush() {
                    Ok(_) => return Ok(()),
                    Err(err) => return Err(err),
                };
            }
            Err(err) => return Err(err),
        }
    }

    Err(Error::new(
        ErrorKind::Other,
        String::from("Failed set polling rate feature to mouse! (Defaulting to 125 hz)"),
    ))
}
