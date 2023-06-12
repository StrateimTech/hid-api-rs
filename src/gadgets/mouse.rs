use std::fs::{File, OpenOptions};
use std::path::Path;

use std::io::{BufWriter, Error, ErrorKind, Read};
use std::sync::RwLock;

use crate::hid;

pub struct Mouse {
    pub mouse_data_buffer: RwLock<Vec<MouseRaw>>,
    pub mouse_state: MouseState,
    pub mouse_device_file: Option<File>,
}

impl Default for Mouse {
    fn default() -> Self {
        Mouse {
            mouse_data_buffer: RwLock::new(Vec::new()),
            mouse_state: MouseState::default(),
            mouse_device_file: None,
        }
    }
}

pub struct MouseState {
    pub left_button: RwLock<bool>,
    pub right_button: RwLock<bool>,
    pub middle_button: RwLock<bool>,

    pub invert_x: RwLock<bool>,
    pub invert_y: RwLock<bool>,
    pub invert_wheel: RwLock<bool>,

    pub sensitivity_multiplier: RwLock<i16>,
}

impl Default for MouseState {
    fn default() -> Self {
        MouseState {
            left_button: RwLock::new(false),
            right_button: RwLock::new(false),
            middle_button: RwLock::new(false),
            invert_x: RwLock::new(false),
            invert_y: RwLock::new(false),
            invert_wheel: RwLock::new(false),
            sensitivity_multiplier: RwLock::new(1),
        }
    }
}

pub struct MouseRaw {
    pub left_button: Option<bool>,
    pub right_button: Option<bool>,
    pub middle_button: Option<bool>,

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
            relative_x: 0,
            relative_y: 0,
            relative_wheel: 0,
        }
    }
}

pub fn attempt_read(mouse: &mut Mouse) -> Result<(), Error>{
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

                let mut relative_x = (i8::from_be_bytes(mouse_buffer[1].to_be_bytes())) as i16;
                if let Ok(invert_x) = mouse.mouse_state.invert_x.read() {
                    if *invert_x {
                        relative_x *= -1;
                    }                 
                }

                let mut relative_y = (i8::from_be_bytes(mouse_buffer[2].to_be_bytes()) * -1) as i16;
                if let Ok(invert_y) = mouse.mouse_state.invert_y.read() {
                    if *invert_y {
                        relative_y *= -1;
                    }                 
                }

                let mut relative_wheel = (i8::from_be_bytes(mouse_buffer[3].to_be_bytes()) * -1) as i16;
                if let Ok(invert_wheel) = mouse.mouse_state.invert_wheel.read() {
                    if *invert_wheel {
                        relative_wheel *= -1;
                    }                 
                }

                if let Ok(mouse_sensitivity) = mouse.mouse_state.sensitivity_multiplier.read() {
                    relative_x *= *mouse_sensitivity;
                    relative_y *= *mouse_sensitivity;
                }

                let raw_mouse = MouseRaw {
                    left_button: Some(left_button),
                    right_button: Some(right_button),
                    middle_button: Some(middle_button),
                    relative_x: relative_x,
                    relative_y: relative_y,
                    relative_wheel: relative_wheel,
                };

                // println!("X: {}, Y: {}, WHEEL: {}, LEFT: {}, RIGHT: {}, MIDDLE: {}", relative_x, relative_y, relative_wheel, left_button, right_button, middle_button);
                push_mouse_event(raw_mouse, mouse)
            }
            return Ok(())
        },
        None => {
            return Err(Error::new(
                ErrorKind::Other,
                String::from("Failed find mouse device file!"),
            ))
        }
    }
}

pub fn push_mouse_event(raw_data: MouseRaw, mouse: &mut Mouse) {
    if let Some(raw_left_button) = raw_data.left_button {
        if let Ok(mut left_button) = mouse.mouse_state.left_button.write() {
            *left_button = raw_left_button;
        }
    }

    if let Some(raw_right_button) = raw_data.right_button {
        if let Ok(mut right_button) = mouse.mouse_state.right_button.write() {
            *right_button = raw_right_button;
        }
    }
    
    if let Some(raw_middle_button) = raw_data.middle_button {
        if let Ok(mut middle_button) = mouse.mouse_state.middle_button.write() {
            *middle_button = raw_middle_button;
        }
    }

    if let Ok(mut buffer) = mouse.mouse_data_buffer.write() {
        buffer.push(raw_data);
    }
}

pub fn push_mouse_event_low_priority(raw_data: MouseRaw, mouse: &mut Mouse) {
    if let Some(raw_left_button) = raw_data.left_button {
        if let Ok(mut left_button) = mouse.mouse_state.left_button.try_write() {
            *left_button = raw_left_button;
        }
    }

    if let Some(raw_right_button) = raw_data.right_button {
        if let Ok(mut right_button) = mouse.mouse_state.right_button.try_write() {
            *right_button = raw_right_button;
        }
    }
    
    if let Some(raw_middle_button) = raw_data.middle_button {
        if let Ok(mut middle_button) = mouse.mouse_state.middle_button.try_write() {
            *middle_button = raw_middle_button;
        }
    }

    if let Ok(mut buffer) = mouse.mouse_data_buffer.try_write() {
        buffer.push(raw_data);
    }
}

pub fn check_mouses(mut mouse_inputs: Vec<String>, mouse_interfaces: &'static mut Vec<Mouse>) {
    loop {
        if mouse_inputs.is_empty() {
            continue;
        }

        for mouse_index in 0..mouse_inputs.len() {
            let mouse_path = &mouse_inputs[mouse_index];
            if Path::exists(Path::new(mouse_path)) {
                let mouse = match OpenOptions::new().write(true).read(true).open(mouse_path) {
                    Ok(result) => result,
                    Err(_) => continue,
                };

                let mut mouse_interface = Mouse {
                    mouse_device_file: Some(mouse),
                    ..Default::default()
                };

                if let Err(err) = hid::write_mouse_scroll_feature(&mut mouse_interface) {
                    panic!("Failed to write mouse scroll feature! {}", err);
                }

                mouse_interfaces.push(mouse_interface);
                mouse_inputs.remove(mouse_index);
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
                if let Err(err) = hid::write_mouse(mouse_raw, gadget_writer){
                    return Err(err);
                }
            }
        }
        Err(err) => {
            return Err(Error::new(
                ErrorKind::Other,
                String::from(format!("Failed to read from mouse data buffer poisoned! ({})", err)),
            ))
        }
    }

    mouse.mouse_data_buffer = RwLock::new(Vec::new());
    Ok(())
}
