use std::{fs::{File, OpenOptions}, io::{BufWriter, Write}};
use std::io::Error;
use std::ops::Deref;

use crate::gadgets::keyboard::KeyboardState;
use crate::gadgets::mouse::MouseRaw;

pub fn open_gadget_device(gadget_device_path: String) -> Result<File, Error> {
    OpenOptions::new()
        .read(false)
        .write(true)
        .append(true)
        .open(gadget_device_path)
}

pub fn write_mouse(raw: &MouseRaw, gadget_writer: &mut BufWriter<File>) -> Result<(), Error> {
    const ID: u8 = 1u8;

    let mut buttons = 0u8;
    if raw.left_button {
        buttons |= 1 << 0;
    }

    if raw.right_button {
        buttons |= 1 << 1;
    }

    if raw.middle_button {
        buttons |= 1 << 2;
    }

    if raw.four_button {
        buttons |= 1 << 3;
    }

    if raw.five_button {
        buttons |= 1 << 4;
    }

    let x_bytes = u16_to_u8s(raw.relative_x as u16);
    let y_bytes = u16_to_u8s(raw.relative_y as u16);
    let wheel_bytes = u16_to_u8s(raw.relative_wheel as u16);

    let formatted_event: [u8; 8] = [
        ID, buttons,
        x_bytes[0], x_bytes[1],
        y_bytes[0], y_bytes[1],
        wheel_bytes[0], wheel_bytes[1]
    ];

    gadget_writer.write(&formatted_event)?;
    gadget_writer.flush()?;

    Ok(())
}

pub fn write_keyboard(
    keyboard_state: &KeyboardState,
    gadget_writer: &mut BufWriter<File>,
) -> Result<(), Error> {
    const ID: u8 = 2u8;

    let mut modifiers_down: Vec<i32> = Vec::new();
    if let Ok(modifiers_down_rwl) = keyboard_state.modifiers_down.read() {
        modifiers_down = modifiers_down_rwl.deref().clone();
    }

    let mut modifier = 0u8;
    for modifier_bit in &modifiers_down {
        modifier |= 1 << *modifier_bit;
    }

    let mut keys_down: Vec<i32> = Vec::new();
    if let Ok(keys_down_rwl) = keyboard_state.keys_down.read() {
        keys_down = keys_down_rwl.deref().clone();
    }

    let mut formatted_event: Vec<u8> = vec![ID];

    match modifiers_down.is_empty() {
        true => formatted_event.push(0),
        false => formatted_event.push(modifier),
    }
    formatted_event.push(0);

    for key_down_index in 1..7 {
        if key_down_index - 1 < keys_down.len() {
            let key_down: i32 = keys_down[key_down_index - 1];
            formatted_event.push(key_down as u8);
        } else {
            formatted_event.push(0);
        }
    }

    gadget_writer.write(&formatted_event)?;
    gadget_writer.flush()?;

    Ok(())
}

fn u16_to_u8s(value: u16) -> [u8; 2] {
    let low_byte = (value >> 8) as u8;
    let high_byte = value as u8;

    [high_byte, low_byte]
}