# hid-api-rs
[![Crates.io](https://img.shields.io/crates/v/hid-api-rs?style=flat-square)](https://crates.io/crates/hid-api-rs) [![docs.rs](https://img.shields.io/docsrs/hid-api-rs?style=flat-square)](https://docs.rs/hid-api-rs/)

Want access to keystrokes & mouse inputs without calling system-level APIs or even before host kernel processing? This library makes it easy by proxying devices through a microcomputer (Raspberry PI 4b & 5), which is then in turn routed into any computer. This allows for Injection, modification, and state viewing of every device proxied.

This implementation is an improvement compared to the [C# version](https://github.com/StrateimTech/HID-API/) as multiple keyboards can function simultaneously.

## Setting Up
1. Ensure that the SBC is connected to the host PC through the OTG port. If you're using a Raspberry Pi 4 Model B or 5, use a USBC-USBA Adapter or USBC cable.
2. Follow [First Installation](#first-installation)
3. Connect a mouse or keyboard (spare if you have one initially to make it easier) to the SBC.
4. Find where your mouse is in `/dev/input/` (it should be `/dev/input/mice/`). Skip this step if you didn't plug a mouse in.
5. If you're using a keyboard, it will differ and should show up in `/dev/input/by-id/` named `...-event-kbd`. Skip this step if you didn't plug a keyboard in.
6. Now, find your gadget device path. It should be `/dev/hidg0` if not, attempt to brute force `hidg0-..x`. This part is important and is required to function.
7. Once you've found the paths, plug them into a project or the [example](https://github.com/StrateimTech/hid-api-rs/blob/master/src/example/bin.rs), build / run and viola! it should pass through your inputs from the microcomputer to the host PC.

## First Installation
Run the following commands on your Raspberry Pi
1. ``echo "dwc2" | sudo tee -a /etc/modules && echo "libcomposite" | sudo tee -a /etc/modules``, these enable OTG Host/Slave drivers & ConfigFS
2. ``echo "dtoverlay=dwc2, dr_mode=peripheral" | /boot/firmware/config.txt``, make sure ``otg_mode=1`` is commented out or removed entirely it won't work otherwise.
3. ``sudo wget -O /usr/bin/example_gadget https://raw.githubusercontent.com/StrateimTech/hid-api-rs/master/example_gadget.sh``
4. ``sudo chmod +x /usr/bin/example_gadget``
5. ``echo "/usr/bin/example_gadget" | sudo tee -a /etc/rc.local``
6. Reboot or run ``sudo /usr/bin/example_gadget``
7. Done, you'll most likely never have to do this again. /dev/hidg0... should auto generate on boot.
8. (Optional) Modify the example gadget to your liking.

If one of these commands didn't work you can follow this external but still relevant [guide](https://www.isticktoit.net/?p=1383) by Tobi.

## Building an [Example](https://github.com/StrateimTech/hid-api-rs/blob/master/src/example/bin-generic.rs)
```
git clone https://github.com/StrateimTech/hid-api-rs
cd ./hid-api-rs
```
Architectures
- 64bit Armv8 - ``aarch64-unknown-linux-gnu``
- 32bit Armv7 - ``armv7-unknown-linux-gnueabihf``
```
cargo build --bin hid_api_example --target armv7-unknown-linux-gnueabihf
```
Once built transfer to pi using preferred method, before running make sure to use elevated permissions since its accessing /dev/ directory.
_(``chmod +x hid_api_example``)_

## Requirements
- Microcomputer / spare computer that supports USB OTG (Raspberry Pi 4 Model B or 5)
- 64bit & 32bit are both supported internally

## Full [Examples](https://github.com/StrateimTech/hid-api-rs/tree/master/src/example)
- ``hid_api_example``, Both keyboard & mouse example code, with state injection.
- ``hid_api_example_mouse``, Mouse only, shows current state of mouse every 500 millis.
- ``hid_api_example_keyboard``, Keyboard only, prints ``hi`` when h is pressed.
- ``hid_api_example_injection``, Injection only no device pass through. It should move the mouse right 25px every second.

Mouse pass-through with state interception:
```rust
use hid_api_rs::{HidSpecification, HidMouse};

pub fn main() {
    hid_api_rs::start_pass_through(HidSpecification {
        mouse_inputs: Some([HidMouse { mouse_path: String::from("/dev/input/mice"), mouse_poll_rate: Some(1000), mouse_side_buttons: true }].to_vec()),
        keyboard_inputs: None,
        gadget_output: String::from("/dev/hidg0")
    }).unwrap();

    loop {
        let mouses = hid_api_rs::get_mouses();

        for mouse in mouses.iter_mut() {
            let mouse_state = *mouse.get_state();

            println!("Left: {}, Right: {}, Middle: {}, Side-4: {}, Side-5: {}", mouse_state.left_button, mouse_state.right_button, mouse_state.middle_button, mouse_state.four_button, mouse_state.five_button);
        }
    }
}
```

Keyboard pass-through with state interception:
```rust
use hid_api_rs::{HidSpecification, gadgets::keyboard::UsbKeyCode};
use std::{time::Duration, thread};

pub fn main() {
    hid_api_rs::start_pass_through(HidSpecification {
        mouse_inputs: None,
        // Use your own keyboard by-id here!
        keyboard_inputs: Some([String::from("/dev/input/by-id/usb-Keychron_K4_Keychron_K4-event-kbd")].to_vec()),
        gadget_output: String::from("/dev/hidg0"),
    }).unwrap();

    loop {
        let keyboard_state = hid_api_rs::get_keyboard();

        if let Ok(keys_down) = &keyboard_state.keys_down.try_read() {
            println!("Keys down: \n {:?}", keys_down.iter().map(|key| format!("{},", UsbKeyCode::from(*key as i16).to_string())).collect::<String>());
        }

        thread::sleep(Duration::from_millis(100))
    }
}
```

# Report descriptor for ConfigFs gadget
```
0x05, 0x01,        // Usage Page (Generic Desktop Ctrls)
0x09, 0x02,        // Usage (Mouse)
0xA1, 0x01,        // Collection (Application)
0x09, 0x01,        //   Usage (Pointer)
0xA1, 0x00,        //   Collection (Physical)
0x85, 0x01,        //     Report ID (1)
0x05, 0x09,        //     Usage Page (Button)
0x19, 0x01,        //     Usage Minimum (0x01)
0x29, 0x05,        //     Usage Maximum (0x05)
0x15, 0x00,        //     Logical Minimum (0)
0x25, 0x01,        //     Logical Maximum (1)
0x95, 0x05,        //     Report Count (5)
0x75, 0x01,        //     Report Size (1)
0x81, 0x02,        //     Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
0x95, 0x01,        //     Report Count (1)
0x75, 0x03,        //     Report Size (3)
0x81, 0x03,        //     Input (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
0x05, 0x01,        //     Usage Page (Generic Desktop Ctrls)
0x09, 0x30,        //     Usage (X)
0x09, 0x31,        //     Usage (Y)
0x16, 0x01, 0x80,  //     Logical Minimum (-32767)
0x26, 0xFF, 0x7F,  //     Logical Maximum (32767)
0x75, 0x10,        //     Report Size (16)
0x95, 0x02,        //     Report Count (2)
0x81, 0x06,        //     Input (Data,Var,Rel,No Wrap,Linear,Preferred State,No Null Position)
0x09, 0x38,        //     Usage (Wheel)
0x15, 0x81,        //     Logical Minimum (-127)
0x25, 0x7F,        //     Logical Maximum (127)
0x75, 0x08,        //     Report Size (8)
0x95, 0x01,        //     Report Count (1)
0x81, 0x06,        //     Input (Data,Var,Rel,No Wrap,Linear,Preferred State,No Null Position)
0xC0,              //   End Collection
0xC0,              // End Collection
0x05, 0x01,        // Usage Page (Generic Desktop Ctrls)
0x09, 0x06,        // Usage (Keyboard)
0xA1, 0x01,        // Collection (Application)
0x85, 0x02,        //   Report ID (2)
0x05, 0x07,        //   Usage Page (Kbrd/Keypad)
0x19, 0xE0,        //   Usage Minimum (0xE0)
0x29, 0xE7,        //   Usage Maximum (0xE7)
0x15, 0x00,        //   Logical Minimum (0)
0x25, 0x01,        //   Logical Maximum (1)
0x75, 0x01,        //   Report Size (1)
0x95, 0x08,        //   Report Count (8)
0x81, 0x02,        //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
0x75, 0x08,        //   Report Size (8)
0x95, 0x01,        //   Report Count (1)
0x81, 0x01,        //   Input (Const,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
0x75, 0x01,        //   Report Size (1)
0x95, 0x03,        //   Report Count (3)
0x05, 0x08,        //   Usage Page (LEDs)
0x19, 0x01,        //   Usage Minimum (Num Lock)
0x29, 0x03,        //   Usage Maximum (Scroll Lock)
0x91, 0x02,        //   Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
0x75, 0x01,        //   Report Size (1)
0x95, 0x05,        //   Report Count (5)
0x91, 0x01,        //   Output (Const,Array,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
0x75, 0x08,        //   Report Size (8)
0x95, 0x06,        //   Report Count (6)
0x15, 0x00,        //   Logical Minimum (0)
0x26, 0xFF, 0x00,  //   Logical Maximum (255)
0x05, 0x07,        //   Usage Page (Kbrd/Keypad)
0x19, 0x00,        //   Usage Minimum (0x00)
0x2A, 0xFF, 0x00,  //   Usage Maximum (0xFF)
0x81, 0x00,        //   Input (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
0xC0,              // End Collection

// 133 bytes
```
## Parsed by https://eleccelerator.com/usbdescreqparser

## HEX
```
0x05 0x01 0x09 0x02 0xA1 0x01 0x09 0x01 0xA1 0x00 0x85 0x01 0x05 0x09 0x19 0x01 0x29 0x03 0x15 0x00 0x25 0x01 0x95 0x03 0x75 0x01 0x81 0x02 0x95 0x01 0x75 0x05 0x81 0x03 0x05 0x01 0x09 0x30 0x09 0x31 0x16 0x01 0x80 0x26 0xFF 0x7F 0x75 0x10 0x95 0x02 0x81 0x06 0x09 0x38 0x15 0x81 0x25 0x7F 0x75 0x08 0x95 0x01 0x81 0x06 0xC0 0xC0 0x05 0x01 0x09 0x06 0xA1 0x01 0x85 0x02 0x05 0x07 0x19 0xe0 0x29 0xe7 0x15 0x00 0x25 0x01 0x75 0x01 0x95 0x08 0x81 0x02 0x75 0x08 0x95 0x01 0x81 0x01 0x75 0x01 0x95 0x03 0x05 0x08 0x19 0x01 0x29 0x03 0x91 0x02 0x75 0x01 0x95 0x05 0x91 0x01 0x75 0x08 0x95 0x06 0x15 0x00 0x26 0xff 0x00 0x05 0x07 0x19 0x00 0x2a 0xff 0x00 0x81 0x00 
```
