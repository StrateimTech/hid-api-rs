use std::{fs::{File, OpenOptions}, io::{BufWriter, Write}};
use std::io;

use bitvec::prelude::Lsb0;
use bitvec::view::BitView;
use bitvec::field::BitField;

use crate::gadgets::mouse::MouseRaw;

static mut GADGET_DEVICE_FILE: Option<File> = None;

pub fn open_gadget_device(gadget_device_path: String) -> Result<&'static mut File, io::Error> {
    unsafe {
        GADGET_DEVICE_FILE = match OpenOptions::new().write(true).append(true).open(gadget_device_path) {
            Ok(result) => Some(result),
            Err(error) => return Err(error)
        };

        if let Some(ref mut gadget) = &mut GADGET_DEVICE_FILE {
            return Ok(gadget)
        }
    }
    panic!("Failed to open device file")
}

pub fn write_mouse(raw: &MouseRaw, gadget_writer: &mut BufWriter<&mut File>) {
    const id: [u8; 1] = [1 as u8];

    let mut buttons = 0u8;
    let bits = buttons.view_bits_mut::<Lsb0>();
    bits.set(1, raw.left_button);
    bits.set(2, raw.right_button);
    bits.set(4, raw.middle_button);
    // buttons = bits.load::<u8>();

    let mouse_x: &[u8] = &[raw.relative_x as u8];
    let mouse_y: &[u8] = &[raw.relative_y as u8];
    let mouse_wheel: &[u8] = &[raw.relative_wheel as u8];

    let mut write_all = || -> Result<(), io::Error> {
        gadget_writer.write(&id)?;
        gadget_writer.write(&[buttons])?;
        gadget_writer.write(mouse_x)?;
        gadget_writer.write(mouse_y)?;
        gadget_writer.write(mouse_wheel)?;
        gadget_writer.flush()?;
        Ok(())
    };

    if let Err(err) = write_all() {
        println!("Failed to write mouse event! {}", err);
    }
}

// TODO: Implement Keyboard
pub fn write_keyboard() {
    // const id: [u8; 1] = [2 as u8];
    todo!();
}