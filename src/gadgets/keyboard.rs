use std::io::{BufWriter, Error, ErrorKind, Read};
use std::{default, fs::File, sync::RwLock};

use crate::hid;

pub struct Keyboard {
    pub keyboard_device_file: Option<File>,
}

pub struct KeyboardState {
    pub keys_down: RwLock<Vec<i32>>,
    pub modifier: RwLock<Option<i32>>,
}

pub enum LinuxKeyCode {
impl Default for KeyboardState {
    fn default() -> Self {
        KeyboardState {
            keys_down: RwLock::new(Vec::new()),
            modifier: RwLock::new(None),
        }
    }
}

pub fn attempt_read(keyboard: &mut Keyboard) -> Result<(), Error> {
    const BUFFER_LENGTH: usize = 24;

    match keyboard.keyboard_device_file {
        Some(ref mut keyboard_file) => {
            let mut keyboard_buffer = [0u8; BUFFER_LENGTH];

            let keyboard_read_length = match keyboard_file.read(&mut keyboard_buffer) {
                Ok(result) => result,
                Err(err) => return Err(err),
            };

            if keyboard_read_length >= BUFFER_LENGTH {
                let key_type = i16::from_ne_bytes([keyboard_buffer[9], keyboard_buffer[10]]);
                let key_code = i16::from_ne_bytes([keyboard_buffer[11], keyboard_buffer[12]]);
                let key_value = i32::from_ne_bytes([
                    keyboard_buffer[13],
                    keyboard_buffer[14],
                    keyboard_buffer[15],
                    keyboard_buffer[16],
                ]);

                // var eventType = (Keyboard.EventType) type;
        }
        None => {
            return Err(Error::new(
                ErrorKind::Other,
                String::from("Failed find mouse device file!"),
            ))
        }
    }

    todo!()
}

pub fn attempt_flush(
    global_keyboard_state: &'static mut KeyboardState,
    gadget_writer: &mut BufWriter<&mut File>,
) -> Result<(), Error> {
    hid::write_keyboard(&global_keyboard_state, gadget_writer)
}

pub fn add_key_down(
    key: i32,
    global_keyboard_state: &'static mut KeyboardState,
) -> Result<(), Error> {
    if let Ok(mut keyboard_state) = global_keyboard_state.keys_down.write() {
        keyboard_state.push(key);

        return Ok(());
    }

    Err(Error::new(
        ErrorKind::Other,
        String::from("Failed to add key down to global key state!"),
    ))
}

// TODO: This will remove all instances of a key
pub fn remove_key_down(
    key: i32,
    global_keyboard_state: &'static mut KeyboardState,
) -> Result<(), Error> {
    if let Ok(mut keyboard_state) = global_keyboard_state.keys_down.write() {
        for key_position in 0..keyboard_state.len() {
            let key_state = &keyboard_state[key_position];
            if key_state == &key {
                keyboard_state.remove(key_position);
            }
        }

        return Ok(());
    }

    Err(Error::new(
        ErrorKind::Other,
        String::from("Failed to remove key from global key state!"),
    ))
}
