use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Error, ErrorKind, Read, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;

use once_cell::sync::Lazy;

use crate::{hid, HidMouse};

pub struct Mouse {
    pub mouse_data_buffer: Vec<MouseRaw>,
    mouse_state: MouseState,
    mouse_device_file: File,

    pub mouse_settings: MouseSettings,
    pub mouse_path: String,

    side_buttons: bool
}

impl Mouse {
    pub fn get_state(&self) -> &MouseState {
        &self.mouse_state
    }
}

#[derive(Copy, Clone, Default)]
pub struct MouseState {
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
    pub four_button: bool,
    pub five_button: bool,
}

pub struct MouseSettings {
    pub invert_x: bool,
    pub invert_y: bool,
    pub invert_wheel: bool,

    pub sensitivity_multiplier: i16,
}

impl Default for MouseSettings {
    fn default() -> Self {
        MouseSettings {
            invert_x: false,
            invert_y: false,
            invert_wheel: false,
            sensitivity_multiplier: 1,
        }
    }
}

#[derive(Default)]
pub struct MouseRaw {
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
    pub four_button: bool,
    pub five_button: bool,

    pub relative_x: i16,
    pub relative_y: i16,
    pub relative_wheel: i16,
}

pub fn attempt_read(mouse: &mut Mouse, gadget_writer: &mut BufWriter<&mut File>) -> Result<(), Error> {
    const BUFFER_LENGTH: usize = 4;
    let mut mouse_buffer = [0u8; BUFFER_LENGTH];

    let mouse_read_length = mouse.mouse_device_file.read(&mut mouse_buffer)?;

    if mouse_read_length >= BUFFER_LENGTH {
        let left_button = mouse_buffer[0] & 0x1 > 0;
        let right_button = mouse_buffer[0] & 0x2 > 0;
        let middle_button = mouse_buffer[0] & 0x4 > 0;

        let mut mouse_four = false;
        let mut mouse_five = false;

        // https://isdaman.com/alsos/hardware/mouse/ps2interface.htm
        if mouse.side_buttons {
            mouse_four = mouse_buffer[3] & 0x10 > 0;
            mouse_five = mouse_buffer[3] & 0x20 > 0;
        }

        let mut relative_y = -i8::from_be_bytes(mouse_buffer[2].to_be_bytes()) as i16;
        let mut relative_x = i8::from_be_bytes(mouse_buffer[1].to_be_bytes()) as i16;

        let mut relative_wheel = match mouse.side_buttons {
            true => {
                let mut z = ((mouse_buffer[3] & 0x8) | (mouse_buffer[3] & 0x4) | (mouse_buffer[3] & 0x2) | (mouse_buffer[3] & 0x1)) as i8;

                if mouse_buffer[3] & 0x8 > 0 {
                    z = (z << 4) >> 4
                }

                -i8::from_be_bytes(z.to_be_bytes()) as i16
            },
            false => -i8::from_be_bytes(mouse_buffer[3].to_be_bytes()) as i16
        };

        if mouse.mouse_settings.invert_x {
            relative_x *= -1;
        }

        if mouse.mouse_settings.invert_y {
            relative_y *= -1;
        }

        if mouse.mouse_settings.invert_wheel {
            relative_wheel *= -1;
        }

        relative_x *= mouse.mouse_settings.sensitivity_multiplier;
        relative_y *= mouse.mouse_settings.sensitivity_multiplier;

        let raw_mouse = MouseRaw {
            left_button,
            right_button,
            middle_button,
            four_button: mouse_four,
            five_button: mouse_five,
            relative_x,
            relative_y,
            relative_wheel,
        };

        return push_mouse_event(raw_mouse, Some(mouse), gadget_writer);
    }

    Ok(())
}

pub fn push_mouse_event(raw_data: MouseRaw, mouse: Option<&mut Mouse>, gadget_writer: &mut BufWriter<&mut File>) -> Result<(), Error> {
    if let Some(mouse) = mouse {
        let new_state = MouseState {
            left_button: raw_data.left_button,
            right_button: raw_data.right_button,
            middle_button: raw_data.middle_button,
            four_button: raw_data.four_button,
            five_button: raw_data.five_button,
        };
        mouse.mouse_state = new_state;
    }

    hid::write_mouse(&raw_data, gadget_writer)
}

pub fn check_mouses(mouse_inputs: &Vec<HidMouse>, mouse_interfaces: &'static mut Lazy<Vec<Mouse>>) {
    for mouse_input in mouse_inputs {
        if mouse_interfaces.iter().any(|mouse| mouse.mouse_path == mouse_input.mouse_path) {
            thread::sleep(Duration::from_millis(1));
            continue;
        }

        let mouse_path = Path::new(&mouse_input.mouse_path);
        if Path::exists(mouse_path) {
            let mouse = match OpenOptions::new()
                .write(true)
                .read(true)
                .open(mouse_path)
            {
                Ok(result) => result,
                Err(_) => continue,
            };

            let mut mouse_interface = Mouse {
                mouse_data_buffer: Vec::new(),
                mouse_state: MouseState::default(),
                mouse_device_file: mouse,
                mouse_settings: MouseSettings::default(),
                mouse_path: mouse_input.mouse_path.clone(),
                side_buttons: mouse_input.mouse_side_buttons
            };

            // https://wiki.osdev.org/PS/2_Mouse#Mouse_Extensions
            let mut mouse_feature: [u8; 6] = [0xf3, 200, 0xf3, 100, 0xf3, 80];
            if mouse_input.mouse_side_buttons {
                mouse_feature = [0xf3, 200, 0xf3, 200, 0xf3, 80];
            }

            _ = write_feature(&mut mouse_interface, mouse_feature);

            mouse_interfaces.push(mouse_interface);
        }
    }
}

pub fn write_feature(mouse: &mut Mouse, feature: [u8; 6]) -> Result<(), Error> {
    if mouse.mouse_device_file.write_all(&feature).is_err() {
        return Err(Error::new(
            ErrorKind::Other,
            String::from("Failed write magic scroll feature to mouse!"),
        ));
    }

    Ok(())
}

pub fn write_poll_rate(mouse: &mut Mouse, poll_rate: i32) -> Result<(), Error> {
    let mut poll_rate_packet: Vec<u8> = Vec::new();
    poll_rate_packet.push(0xf3);
    poll_rate_packet.append(&mut poll_rate.to_be_bytes().to_vec());

    if mouse.mouse_device_file.write_all(&poll_rate_packet).is_err() {
        return Err(Error::new(
            ErrorKind::Other,
            String::from("Failed to set poll rate to mouse! (Defaulting to 125 hz or last mouse profile)"),
        ));
    }

    Ok(())
}