use std::path::Path;
use std::fs::{OpenOptions, File};

use std::io::{Read, Error, BufWriter};
use std::sync::RwLock;

use crate::hid;

pub struct Mouse {
    pub mouse_data_buffer: RwLock<Vec<MouseRaw>>,
    pub mouse_config: MouseConfig,
    pub mouse_device_file: Option<File>
}

impl Default for Mouse {
    fn default() -> Self {
        Mouse {
            mouse_data_buffer: RwLock::new(Vec::new()),
            mouse_config: MouseConfig::default(),
            mouse_device_file: None
        }
    }
}

pub struct MouseConfig {
    pub invert_mouse_x: bool,
    pub invert_mouse_y: bool,
    pub invert_mouse_wheel: bool,

    pub sensitivity_multiplier: i8
}

impl Default for MouseConfig {
    fn default() -> Self {
        MouseConfig {
            invert_mouse_x: false, 
            invert_mouse_y: false,
            invert_mouse_wheel: false,
            sensitivity_multiplier: 1
        }
    }
}

pub struct MouseRaw {
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,

    pub relative_x: i8,
    pub relative_y: i8,
    pub relative_wheel: i8
}

impl Default for MouseRaw {
    fn default() -> Self {
        MouseRaw {
            left_button: false,
            right_button: false,
            middle_button: false,
            relative_x: 0,
            relative_y: 0,
            relative_wheel: 0
        }
    }
}

pub fn attempt_read(mouse: &mut Mouse) {
    const BUFFER_LENGTH: usize = 4;

    if let Some(ref mut mouse_file) = mouse.mouse_device_file {
        let mut mouse_buffer = [0u8; BUFFER_LENGTH];

        let mouse_read_length = match mouse_file.read(&mut mouse_buffer) {
            Ok(result) => result,
            Err(_) => return
        };

        if mouse_read_length >= BUFFER_LENGTH {
            let left_button = mouse_buffer[0] & 0x1 > 0;
            let right_button = mouse_buffer[0] & 0x2 > 0;
            let middle_button =  mouse_buffer[0] & 0x4 > 0;

            let relative_x = match (i8::from_be_bytes(mouse_buffer[1].to_be_bytes()), mouse.mouse_config.invert_mouse_x) {
                (value, true) => value * -1,
                (value, false) => value
            };
            let relative_y = match (i8::from_be_bytes(mouse_buffer[2].to_be_bytes()), mouse.mouse_config.invert_mouse_y) {
                (value, true) => value * -1,
                (value, false) => value
            };
            let relative_wheel = match (i8::from_be_bytes(mouse_buffer[3].to_be_bytes()), mouse.mouse_config.invert_mouse_wheel) {
                (value, true) => value * -1,
                (value, false) => value
            };

            let raw_mouse = MouseRaw {
                left_button: left_button,
                right_button: right_button,
                middle_button: middle_button,
                relative_x: relative_x,
                relative_y: relative_y,
                relative_wheel: relative_wheel
            };

            push_mouse_event(raw_mouse, mouse)
        }
    }
}

pub fn push_mouse_event(raw_data: MouseRaw, mouse: &mut Mouse) {
    if let Ok(mut buffer) = mouse.mouse_data_buffer.try_write() {
        buffer.push(raw_data);
    }
}

pub fn check_mouses(mut mouse_inputs: Vec<String>, mouse_interfaces: &'static mut Vec<Mouse>) {
    loop {
        for mouse_index in 0..mouse_inputs.len() {
            let mouse_path = &mouse_inputs[mouse_index];
            if Path::exists(Path::new(mouse_path)) {
                let mouse_interface = Mouse::default(); 
    
                let mouse = match OpenOptions::new().write(true).read(true).open(mouse_path) {
                    Ok(result) => result,
                    Err(_) => continue
                };
    
                Mouse {
                    mouse_device_file: Some(mouse),
                    ..Default::default()
                };

                mouse_interfaces.push(mouse_interface);
                mouse_inputs.remove(mouse_index);
            }
        }
    }
}

// TODO: This could possible (very low I think) to drop or lose mouse events
pub fn attempt_flush(mouse: &mut Mouse, gadget_writer: &mut BufWriter<&mut File>) -> Result<(), ()> {
    match mouse.mouse_data_buffer.try_read() {
        Ok(mouse_buffer) => {
            for mouse_raw in mouse_buffer.iter() {
                hid::write_mouse(mouse_raw, gadget_writer)
            }
        }
        Err(_) => return Err(())
    }
    mouse.mouse_data_buffer = RwLock::new(Vec::new());

    Ok(())
}