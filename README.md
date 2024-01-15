# hid-api-rs
This is a rewritten version of [HID-API's](https://github.com/StrateimTech/HID-API/) C# implementation, now in Rust!

[![Crates.io](https://img.shields.io/crates/v/hid-api-rs?style=flat-square)](https://crates.io/crates/hid-api-rs) [![docs.rs](https://img.shields.io/docsrs/hid-api-rs?style=flat-square)](https://docs.rs/hid-api-rs/)

Want to create cheats without calling system-level APIs for keystrokes and mouse inputs? This library makes it easy by converging your keyboards and mice directly into a microcomputer (Raspberry Pi), which is then routed into any other computer effectively passing through. This allows for independent external calls from the main computer, bypassing kernel-level anti-cheats. It's also capable of injecting keystrokes and mouse movements.

This implementation is an improvement compared to the [C# version](https://github.com/StrateimTech/HID-API/) as the modifier bit field is reported correctly and multiple keyboards can function simultaneously.

## Setting Up
If you're unfamiliar with the process of setting this up, follow these steps:

1. Follow this [guide](https://www.isticktoit.net/?p=1383) to set up a gadget that includes keyboard and mouse elements **however it will not function with this library**. Here's a completed [example script](https://github.com/StrateimTech/hid-api-rs/blob/master/example_gadget.sh/) that has been tested to work with this library.
2. Ensure that the slave is connected to the master host PC (this should show up as a single device with both keyboard and mouse protocols). If you're using a Raspberry Pi 4 Model B, use a USBC-USBA Adapter or USBC cable.
3. Connect a mouse or keyboard (spare if you have one initially to make it easier) to the slave.
4. Find where your mouse is in `/dev/input/` (it should be `/dev/input/mice/`). Skip this step if you didn't plug a mouse in.
5. If you're using a keyboard, it will differ and should show up in `/dev/input/by-id/` named `...-event-kbd`. Skip this step if you didn't plug a keyboard in.
6. Now, find your gadget device path. It should be `/dev/hidg0` if not, attempt to brute force `hidg0-..x`. This part is important and is required to function.
7. Once you've found the paths, plug them into a project or the [example](https://github.com/StrateimTech/hid-api-rs/blob/master/src/example/bin.rs), build / run and viola! it should passthrough your inputs from the slave microcomputer to the master PC.

## Building the [Example](https://github.com/StrateimTech/hid-api-rs/blob/master/src/example/bin-generic.rs) for a Pi4 Model b (Only model that supports USB OTG)
```
git clone https://github.com/StrateimTech/hid-api-rs
cd ./hid-api-rs
cargo build --bin hid_api_example --target armv7-unknown-linux-gnueabihf
```
Once built transfer to pi using preferred method, before running make sure to use elevated permissions since its accessing /dev/ directory.

## Examples
- ``hid_api_example``, Has both keyboard & mouse showcase code
- ``hid_api_example_mouse``, Has mouse only and shows current state of mouse every 500 millis.

## Requirements
- Microcomputer / spare computer that supports USB OTG (Raspberry Pi 4 Model B)
- Keyboard or Mouse

# Report descriptor for configfs gadget
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
