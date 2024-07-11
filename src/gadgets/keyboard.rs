use std::{fs::File, sync::RwLock, thread};
use std::fmt::{self, Debug};
use std::fs::OpenOptions;
use std::io::{BufWriter, Error, ErrorKind, Read, Write};
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use num_enum::FromPrimitive;
use strum_macros::EnumString;

use crate::hid;

pub struct Keyboard {
    pub keyboard_device_file: File,
    pub keyboard_path: String,
}

pub struct KeyboardState {
    pub keys_down: RwLock<Vec<i32>>,
    pub modifiers_down: RwLock<Vec<i32>>,
}

impl Default for KeyboardState {
    fn default() -> Self {
        KeyboardState {
            keys_down: RwLock::new(Vec::new()),
            modifiers_down: RwLock::new(Vec::new()),
        }
    }
}

// https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h
#[derive(FromPrimitive, Debug, Clone, Copy)]
#[repr(i16)]
enum LinuxKeyCode {
    #[num_enum(default)]
    KEYRESERVED = 0,
    KEYESC = 1,
    KEY1 = 2,
    KEY2 = 3,
    KEY3 = 4,
    KEY4 = 5,
    KEY5 = 6,
    KEY6 = 7,
    KEY7 = 8,
    KEY8 = 9,
    KEY9 = 10,
    KEY0 = 11,
    KEYMINUS = 12,
    KEYEQUAL = 13,
    KEYBACKSPACE = 14,
    KEYTAB = 15,
    KEYQ = 16,
    KEYW = 17,
    KEYE = 18,
    KEYR = 19,
    KEYT = 20,
    KEYY = 21,
    KEYU = 22,
    KEYI = 23,
    KEYO = 24,
    KEYP = 25,
    KEYLEFTBRACE = 26,
    KEYRIGHTBRACE = 27,
    KEYENTER = 28,
    KEYLEFTCTRL = 29,
    KEYA = 30,
    KEYS = 31,
    KEYD = 32,
    KEYF = 33,
    KEYG = 34,
    KEYH = 35,
    KEYJ = 36,
    KEYK = 37,
    KEYL = 38,
    KEYSEMICOLON = 39,
    KEYAPOSTROPHE = 40,
    KEYGRAVE = 41,
    KEYLEFTSHIFT = 42,
    KEYBACKSLASH = 43,
    KEYZ = 44,
    KEYX = 45,
    KEYC = 46,
    KEYV = 47,
    KEYB = 48,
    KEYN = 49,
    KEYM = 50,
    KEYCOMMA = 51,
    KEYDOT = 52,
    KEYSLASH = 53,
    KEYRIGHTSHIFT = 54,
    KEYKPASTERISK = 55,
    KEYLEFTALT = 56,
    KEYSPACE = 57,
    KEYCAPSLOCK = 58,
    // KEYF1 = 59,
    // KEYF2 = 60,
    // KEYF3 = 61,
    // KEYF4 = 62,
    // KEYF5 = 63,
    // KEYF6 = 64,
    // KEYF7 = 65,
    // KEYF8 = 66,
    // KEYF9 = 67,
    // KEYF10 = 68,
    KEYNUMLOCK = 69,
    KEYSCROLLLOCK = 70,
    KEYKP7 = 71,
    KEYKP8 = 72,
    KEYKP9 = 73,
    KEYKPMINUS = 74,
    KEYKP4 = 75,
    KEYKP5 = 76,
    KEYKP6 = 77,
    KEYKPPLUS = 78,
    KEYKP1 = 79,
    KEYKP2 = 80,
    KEYKP3 = 81,
    KEYKP0 = 82,
    KEYKPDOT = 83,

    KEYZENKAKUHANKAKU = 85,
    KEY102ND = 86,
    // KEYF11 = 87,
    // KEYF12 = 88,
    KEYRO = 89,
    KEYKATAKANA = 90,
    KEYHIRAGANA = 91,
    KEYHENKAN = 92,
    KEYKATAKANAHIRAGANA = 93,
    KEYMUHENKAN = 94,
    KEYKPJPCOMMA = 95,
    KEYKPENTER = 96,
    KEYRIGHTCTRL = 97,
    KEYKPSLASH = 98,
    KEYSYSRQ = 99,
    KEYRIGHTALT = 100,
    KEYLINEFEED = 101,
    KEYHOME = 102,
    KEYUP = 103,
    KEYPAGEUP = 104,
    KEYLEFT = 105,
    KEYRIGHT = 106,
    KEYEND = 107,
    KEYDOWN = 108,
    KEYPAGEDOWN = 109,
    KEYINSERT = 110,
    KEYDELETE = 111,
    KEYMACRO = 112,
    KEYPOWER = 116,
    KEYKPEQUAL = 117,
    KEYKPPLUSMINUS = 118,
    KEYPAUSE = 119,

    KEYKPCOMMA = 121,

    KEYHANJA = 123,
    KEYYEN = 124,
    KEYLEFTMETA = 125,
    KEYRIGHTMETA = 126,
    KEYCOMPOSE = 127,

    KEYSTOP = 128,
    KEYAGAIN = 129,
    KEYPROPS = 130,
    KEYUNDO = 131,
    KEYFRONT = 132,
    KEYCOPY = 133,
    KEYOPEN = 134,
    KEYPASTE = 135,
    KEYFIND = 136,
    KEYCUT = 137,
    KEYHELP = 138,
    KEYMENU = 139,
    KEYCALC = 140,
    KEYSETUP = 141,
    KEYSLEEP = 142,
    KEYWAKEUP = 143,
    KEYFILE = 144,
    KEYSENDFILE = 145,
    KEYDELETEFILE = 146,
    KEYXFER = 147,
    KEYPROG1 = 148,
    KEYPROG2 = 149,
    KEYWWW = 150,
    KEYFORWARD = 159,
    KEYCLOSECD = 160,
    KEYEJECTCD = 161,
    KEYEJECTCLOSECD = 162,
    KEYSTOPCD = 166,
    KEYRECORD = 167,
    KEYREWIND = 168,
    KEYPHONE = 169,
    KEYISO = 170,
    KEYCONFIG = 171,
    KEYHOMEPAGE = 172,
    KEYREFRESH = 173,
    KEYEXIT = 174,
    KEYMOVE = 175,
    KEYEDIT = 176,
    KEYSCROLLUP = 177,
    KEYSCROLLDOWN = 178,
    KEYKPLEFTPAREN = 179,
    KEYKPRIGHTPAREN = 180,
    KEYNEW = 181,
    KEYREDO = 182,

    KEYF13 = 183,
    KEYF14 = 184,
    KEYF15 = 185,
    KEYF16 = 186,
    KEYF17 = 187,
    KEYF18 = 188,
    KEYF19 = 189,
    KEYF20 = 190,
    KEYF21 = 191,
    KEYF22 = 192,
    KEYF23 = 193,
    KEYF24 = 194,

    KEYPLAYCD = 200,
    KEYPAUSECD = 201,
    KEYPROG3 = 202,
    KEYPROG4 = 203,
    KEYSUSPEND = 205,
    KEYCLOSE = 206,
    KEYPLAY = 207,
    KEYFASTFORWARD = 208,
    KEYBASSBOOST = 209,
    KEYPRINT = 210,
    KEYHP = 211,
    KEYCAMERA = 212,
    KEYSOUND = 213,
    KEYQUESTION = 214,
    KEYEMAIL = 215,
    KEYCHAT = 216,
    KEYSEARCH = 217,
    KEYCONNECT = 218,
    KEYFINANCE = 219,
    KEYSPORT = 220,
    KEYSHOP = 221,
    KEYALTERASE = 222,
    KEYCANCEL = 223,

    // Multi Media Keys

    // KEYBRIGHTNESSDOWN = 224,
    // KEYBRIGHTNESSUP = 225,
    // KEYSCALE = 120,
    // KEYDASHBOARD = 204,
    // KEYKBDILLUMDOWN = 229,
    // KEYKBDILLUMUP = 230,
    // KEYPREVIOUSSONG = 165,
    // KEYPLAYPAUSE = 164,
    // KEYNEXTSONG = 163,
    // KEYMUTE = 113,
    // KEYVOLUMEDOWN = 114,
    // KEYVOLUMEUP = 115,

    KEYF1 = 224,
    KEYF2 = 225,
    KEYF3 = 120,
    KEYF4 = 204,
    KEYF5 = 229,
    KEYF6 = 230,
    KEYF7 = 165,
    KEYF8 = 164,
    KEYF9 = 163,
    KEYF10 = 113,
    KEYF11 = 114,
    KEYF12 = 115,

    KEYMEDIA = 226,

    KEYSWITCHVIDEOMODE = 227,
    KEYKBDILLUMTOGGLE = 228,

    KEYSEND = 231,
    KEYREPLY = 232,
    KEYFORWARDMAIL = 233,
    KEYSAVE = 234,
    KEYDOCUMENTS = 235,

    KEYBATTERY = 236,

    KEYBLUETOOTH = 237,
    KEYWLAN = 238,
    KEYUWB = 239,

    KEYUNKNOWN = 240,

    KEYVIDEONEXT = 241,
    KEYVIDEOPREV = 242,
    KEYBRIGHTNESSCYCLE = 243,
    KEYBRIGHTNESSZERO = 244,
    KEYDISPLAYOFF = 245,
}

impl fmt::Display for LinuxKeyCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// USB Hid Key Codes
#[derive(FromPrimitive, Clone, Copy, EnumString, Debug)]
#[repr(i16)]
pub enum UsbKeyCode {
    #[num_enum(default)]
    KEYRESERVED = 0x00,
    KEYA = 0x04,
    KEYB = 0x05,
    KEYC = 0x06,
    KEYD = 0x07,
    KEYE = 0x08,
    KEYF = 0x09,
    KEYG = 0x0a,
    KEYH = 0x0b,
    KEYI = 0x0c,
    KEYJ = 0x0d,
    KEYK = 0x0e,
    KEYL = 0x0f,
    KEYM = 0x10,
    KEYN = 0x11,
    KEYO = 0x12,
    KEYP = 0x13,
    KEYQ = 0x14,
    KEYR = 0x15,
    KEYS = 0x16,
    KEYT = 0x17,
    KEYU = 0x18,
    KEYV = 0x19,
    KEYW = 0x1a,
    KEYX = 0x1b,
    KEYY = 0x1c,
    KEYZ = 0x1d,

    KEY1 = 0x1e,
    KEY2 = 0x1f,
    KEY3 = 0x20,
    KEY4 = 0x21,
    KEY5 = 0x22,
    KEY6 = 0x23,
    KEY7 = 0x24,
    KEY8 = 0x25,
    KEY9 = 0x26,
    KEY0 = 0x27,

    KEYENTER = 0x28,
    KEYESC = 0x29,
    KEYBACKSPACE = 0x2a,
    KEYTAB = 0x2b,
    KEYSPACE = 0x2c,
    KEYMINUS = 0x2d,
    KEYEQUAL = 0x2e,
    KEYLEFTBRACE = 0x2f,
    KEYRIGHTBRACE = 0x30,
    KEYBACKSLASH = 0x31,
    KEYHASHTILDE = 0x32,
    KEYSEMICOLON = 0x33,
    KEYAPOSTROPHE = 0x34,
    KEYGRAVE = 0x35,
    KEYCOMMA = 0x36,
    KEYDOT = 0x37,
    KEYSLASH = 0x38,
    KEYCAPSLOCK = 0x39,

    KEYF1 = 0x3a,
    KEYF2 = 0x3b,
    KEYF3 = 0x3c,
    KEYF4 = 0x3d,
    KEYF5 = 0x3e,
    KEYF6 = 0x3f,
    KEYF7 = 0x40,
    KEYF8 = 0x41,
    KEYF9 = 0x42,
    KEYF10 = 0x43,
    KEYF11 = 0x44,
    KEYF12 = 0x45,

    KEYSYSRQ = 0x46,
    KEYSCROLLLOCK = 0x47,
    KEYPAUSE = 0x48,
    KEYINSERT = 0x49,
    KEYHOME = 0x4a,
    KEYPAGEUP = 0x4b,
    KEYDELETE = 0x4c,
    KEYEND = 0x4d,
    KEYPAGEDOWN = 0x4e,
    KEYRIGHT = 0x4f,
    KEYLEFT = 0x50,
    KEYDOWN = 0x51,
    KEYUP = 0x52,

    KEYNUMLOCK = 0x53,
    KEYKPSLASH = 0x54,
    KEYKPASTERISK = 0x55,
    KEYKPMINUS = 0x56,
    KEYKPPLUS = 0x57,
    KEYKPENTER = 0x58,
    KEYKP1 = 0x59,
    KEYKP2 = 0x5a,
    KEYKP3 = 0x5b,
    KEYKP4 = 0x5c,
    KEYKP5 = 0x5d,
    KEYKP6 = 0x5e,
    KEYKP7 = 0x5f,
    KEYKP8 = 0x60,
    KEYKP9 = 0x61,
    KEYKP0 = 0x62,
    KEYKPDOT = 0x63,

    KEY102ND = 0x64,
    KEYCOMPOSE = 0x65,
    KEYPOWER = 0x66,
    KEYKPEQUAL = 0x67,

    KEYF13 = 0x68,
    KEYF14 = 0x69,
    KEYF15 = 0x6a,
    KEYF16 = 0x6b,
    KEYF17 = 0x6c,
    KEYF18 = 0x6d,
    KEYF19 = 0x6e,
    KEYF20 = 0x6f,
    KEYF21 = 0x70,
    KEYF22 = 0x71,
    KEYF23 = 0x72,
    KEYF24 = 0x73,

    KEYOPEN = 0x74,
    KEYHELP = 0x75,
    KEYPROPS = 0x76,
    KEYFRONT = 0x77,
    KEYSTOP = 0x78,
    KEYAGAIN = 0x79,
    KEYUNDO = 0x7a,
    KEYCUT = 0x7b,
    KEYCOPY = 0x7c,
    KEYPASTE = 0x7d,
    KEYFIND = 0x7e,
    KEYMUTE = 0x7f,
    KEYVOLUMEUP = 0x80,
    KEYVOLUMEDOWN = 0x81,
}

impl fmt::Display for UsbKeyCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, FromPrimitive, Debug)]
#[repr(i16)]
enum EventType {
    #[num_enum(default)]
    EvSyn,
    EvKey,
    EvRel,
    EvAbs,
    EvMsc,
    EvSw,
    EvLed,
    EvSnd,
    EvRep,
    EvFf,
    EvPwr,
    EvFfStatus,
}

#[derive(FromPrimitive, Debug)]
#[repr(i32)]
enum KeyState {
    KeyUp,
    #[num_enum(default)]
    KeyDown,
    KeyHold,
}

#[derive(FromPrimitive, EnumString, Clone, Copy, Debug)]
#[repr(i32)]
pub enum KeyCodeModifier {
    #[num_enum(default)]
    KEYLEFTCTRL = 0,
    KEYLEFTSHIFT = 1,
    KEYLEFTALT = 2,
    KEYLEFTMETA = 3,
    KEYRIGHTCTRL = 4,
    KEYRIGHTSHIFT = 5,
    KEYRIGHTALT = 6,
    KEYRIGHTMETA = 7,
}

impl fmt::Display for KeyCodeModifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(target_pointer_width = "64")]
const BUFFER_LENGTH: usize = 24;

#[cfg(target_pointer_width = "32")]
const BUFFER_LENGTH: usize = 12;

pub fn attempt_read(
    keyboard: &mut Keyboard,
    global_keyboard_state: &'static mut KeyboardState,
    writer: &mut BufWriter<File>
) -> Result<(), Error> {
    let mut keyboard_buffer = [0u8; BUFFER_LENGTH];

    let keyboard_read_length = keyboard.keyboard_device_file.read(&mut keyboard_buffer)?;

    if keyboard_read_length >= BUFFER_LENGTH {
        let mut offset = match BUFFER_LENGTH {
            24 => 16,
            12 => 8,
            _ => 8,
        };

        let key_type = i16::from_ne_bytes([keyboard_buffer[offset], keyboard_buffer[{ offset += 1; offset }]]);
        let key_code = i16::from_ne_bytes([keyboard_buffer[{ offset += 1; offset }], keyboard_buffer[{ offset += 1; offset }]]);
        let key_value = i32::from_ne_bytes([
            keyboard_buffer[{ offset += 1; offset }],
            keyboard_buffer[{ offset += 1; offset }],
            keyboard_buffer[{ offset += 1; offset }],
            keyboard_buffer[{ offset += 1; offset }],
        ]);

        let linux_code: LinuxKeyCode = LinuxKeyCode::from(key_code);
        let usb_code = match UsbKeyCode::from_str(LinuxKeyCode::to_string(&linux_code).as_str()) {
            Ok(code) => Some(code),
            Err(_) => None
        };

        let event_type: EventType = EventType::from(key_type);
        let key_state: KeyState = KeyState::from(key_value);

        let key_modifier = match KeyCodeModifier::from_str(LinuxKeyCode::to_string(&linux_code).as_str()) {
            Ok(modifier) => Some(modifier),
            Err(_) => None
        };

        if event_type == EventType::EvKey {
            let mut result: Result<(), Error> = Ok(());
            
            match key_state {
                KeyState::KeyDown | KeyState::KeyHold => {
                    match key_modifier {
                        Some(modifier) => {
                            result = add_generic_down(modifier as i32, &global_keyboard_state.modifiers_down);
                        }
                        None => {
                            if let Some(code) = usb_code {
                                result = add_generic_down(code as i32, &global_keyboard_state.keys_down);
                            }
                        }
                    }
                }
                KeyState::KeyUp => {
                    match key_modifier {
                        Some(modifier) => {
                            result = remove_generic_down(modifier as i32, &global_keyboard_state.modifiers_down);
                        }
                        None => {
                            if let Some(code) = usb_code {
                                result = remove_generic_down(code as i32, &global_keyboard_state.keys_down);
                            }
                        }
                    }
                }
            };

            attempt_flush(global_keyboard_state, writer)?;

            return result;
        }
    }

    Ok(())
}

pub fn attempt_flush(
    global_keyboard_state: &'static mut KeyboardState,
    gadget_writer: &mut BufWriter<File>,
) -> Result<(), Error> {
    hid::write_keyboard(global_keyboard_state, gadget_writer)
}

pub fn add_generic_down(key: i32, key_vec: &RwLock<Vec<i32>>) -> Result<(), Error> {
    if let Ok(generic_keyvec) = key_vec.read() {
        if generic_keyvec.contains(&(key)) {
            return Ok(());
        }
    }

    if let Ok(mut generic_keyvec_writer) = key_vec.write() {
        generic_keyvec_writer.push(key);

        return Ok(());
    }

    Err(Error::new(
        ErrorKind::Other,
        String::from("Failed to push key to generic keyvec (Modifier / Key)"),
    ))
}

pub fn remove_generic_down(key: i32, key_vec: &RwLock<Vec<i32>>) -> Result<(), Error> {
    if let Ok(mut key_vec) = key_vec.write() {
        key_vec.retain(|k| *k != key);
        return Ok(());
    }

    Err(Error::new(
        ErrorKind::Other,
        String::from("Failed to remove key from generic keyvec (Modifier / Key)"),
    ))
}

pub fn check_keyboards(keyboard_inputs: &Vec<String>, keyboard_interfaces: &'static mut Vec<Keyboard>) {
    for keyboard_input in keyboard_inputs {
        if keyboard_interfaces.iter().any(|keyboard_interface| &keyboard_interface.keyboard_path == keyboard_input) {
            thread::sleep(Duration::from_millis(1));
            continue;
        }

        let keyboard_path = Path::new(&keyboard_input);

        if Path::exists(keyboard_path) {
            let keyboard = match OpenOptions::new()
                .write(true)
                .read(true)
                .open(keyboard_path)
            {
                Ok(result) => result,
                Err(_) => {
                    continue;
                }
            };

            let mut keyboard_interface = Keyboard {
                keyboard_device_file: keyboard,
                keyboard_path: keyboard_input.clone(),
            };

            _ = write_scancode_set(&mut keyboard_interface);

            keyboard_interfaces.push(keyboard_interface);
        }
    }
}

pub fn is_key_down(
    key: UsbKeyCode,
    global_keyboard_state: &&'static mut KeyboardState,
) -> bool {
    if let Ok(keyboard_state) = global_keyboard_state.keys_down.try_read() {
        return keyboard_state.contains(&(key as i32));
    }

    false
}

pub fn is_modifier_down(
    modifier: KeyCodeModifier,
    global_keyboard_state: &&'static mut KeyboardState,
) -> bool {
    if let Ok(modifiers_down) = global_keyboard_state.modifiers_down.try_read() {
        return modifiers_down.contains(&(modifier as i32));
    }

    false
}

// https://wiki.osdev.org/PS/2_Keyboard
pub fn write_scancode_set(keyboard: &mut Keyboard) -> Result<(), Error> {
    let keyboard_scancode_packet: [u8; 2] = [0xF0, 2];

    match keyboard.keyboard_device_file.write_all(&keyboard_scancode_packet) {
        Ok(_) => {
            match keyboard.keyboard_device_file.flush() {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err)
    }
}