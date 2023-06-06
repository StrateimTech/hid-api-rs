use std::io::Error;
use std::ops::Deref;
use std::{
    fs::{File, OpenOptions},
    io,
    io::{BufWriter, Write},
};

use bitvec::prelude::Lsb0;
use bitvec::view::BitView;

use crate::gadgets::keyboard::{KeyCodeModifier, KeyboardState};
use crate::gadgets::mouse::{Mouse, MouseRaw};

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
    bits.set(0, raw.left_button);
    bits.set(1, raw.right_button);
    bits.set(2, raw.middle_button);

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

pub fn write_mouse_scroll_feature(mouse: &mut Mouse) -> Result<(), Error> {
    let mouse_scroll: [u8; 6] = [0xf3, 200, 0xf3, 100, 0xf3, 80];
    if let Some(ref mut mouse_data_buffer) = mouse.mouse_device_file {
        match mouse_data_buffer.write(&mouse_scroll) {
            Ok(_) => return Ok(()),
            Err(err) => return Err(err)
        };
    }
    Ok(())
}

pub fn write_keyboard(
    keyboard_state: &KeyboardState,
    gadget_writer: &mut BufWriter<&mut File>,
) -> Result<(), Error> {
    const ID: [u16; 1] = [2 as u16];

    let mut modifier: Option<KeyCodeModifier> = None;
    if let Ok(modifier_rwl) = keyboard_state.modifier.try_read() {
        modifier = modifier_rwl.deref().clone();
    }

    let mut keys_down: Vec<i32> = Vec::new();
    if let Ok(keys_down_rwl) = keyboard_state.keys_down.try_read() {
        keys_down = keys_down_rwl.deref().clone();
    }

    let mut formatted_event: Vec<u8> = [as_u8_slice(&ID)].concat();

    if let Some(ref md) = modifier {
        formatted_event.push(*md as u8)
    }

    for key_down_index in 0..keys_down.len().min(6) {
        let key_down = keys_down[key_down_index];
        formatted_event.push(key_down as u8);
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