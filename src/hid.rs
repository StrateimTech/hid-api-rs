use std::io::Error;
use std::ops::Deref;
use std::{
    fs::{File, OpenOptions},
    io,
    io::{BufWriter, Write},
};

use bitvec::prelude::Lsb0;
use bitvec::view::BitView;

use crate::gadgets::keyboard::KeyboardState;
use crate::gadgets::mouse::MouseRaw;

static mut GADGET_DEVICE_FILE: Option<File> = None;

pub fn open_gadget_device(gadget_device_path: String) -> Result<&'static mut File, io::Error> {
    unsafe {
        GADGET_DEVICE_FILE = match OpenOptions::new()
            .write(true)
            .append(true)
            .open(gadget_device_path)
        {
            Ok(result) => Some(result),
            Err(error) => return Err(error),
        };

        if let Some(ref mut gadget) = &mut GADGET_DEVICE_FILE {
            return Ok(gadget);
        }
    }
    panic!("Failed to open device file")
}

pub fn write_mouse(raw: &MouseRaw, gadget_writer: &mut BufWriter<&mut File>) -> Result<(), Error> {
    const ID: [u8; 1] = [1 as u8];

    let mut buttons = 0u8;
    let bits = buttons.view_bits_mut::<Lsb0>();
    if let Some(left_button) = raw.left_button {
        bits.set(0, left_button);
    }

    if let Some(right_button) = raw.right_button {
        bits.set(1, right_button);
    }

    if let Some(middle_button) = raw.middle_button {
        bits.set(2, middle_button);
    }

    if let Some(four_button) = raw.four_button {
        bits.set(3, four_button);
    }

    if let Some(five_button) = raw.five_button {
        bits.set(4, five_button);
    }

    let mouse_x: &[u16] = &[raw.relative_x as u16];
    let mouse_y: &[u16] = &[raw.relative_y as u16];
    let mouse_wheel: &[u16] = &[raw.relative_wheel as u16];

    let formatted_event = [
        &ID,
        &[buttons],
        as_u8_slice(mouse_x),
        as_u8_slice(mouse_y),
        as_u8_slice(mouse_wheel),
    ]
    .concat();

    if let Err(err) = gadget_writer.write(&formatted_event) {
        return Err(err);
    }

    if let Err(err) = gadget_writer.flush() {
        return Err(err);
    };

    Ok(())
}

pub fn write_keyboard(
    keyboard_state: &KeyboardState,
    gadget_writer: &mut BufWriter<&mut File>,
) -> Result<(), Error> {
    const ID: u8 = 2;

    let mut modifiers_down: Vec<i32> = Vec::new();
    if let Ok(modifiers_down_rwl) = keyboard_state.modifiers_down.read() {
        modifiers_down = modifiers_down_rwl.deref().clone();
    }

    let mut modifier = 0u8;
    let bits = modifier.view_bits_mut::<Lsb0>();
    for modifier in &modifiers_down {
        bits.set(modifier.clone() as usize, true);
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

    if let Err(err) = gadget_writer.write(&formatted_event) {
        return Err(err);
    }

    if let Err(err) = gadget_writer.flush() {
        return Err(err);
    };

    Ok(())
}

fn as_u8_slice(slice: &[u16]) -> &[u8] {
    let len = 2 * slice.len();

    let ptr = slice.as_ptr().cast::<u8>();
    unsafe { std::slice::from_raw_parts(ptr, len) }
}
