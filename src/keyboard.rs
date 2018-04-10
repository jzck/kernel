extern crate core;

use cpuio;
use vga;

const MAX_KEYS: usize = 59;
const KEYMAP_US: [[u8; 2]; MAX_KEYS] = [
    *b"\0\0",
    *b"\0\0",//escape
    *b"1!",
    *b"2@",
    *b"3#",
    *b"4$",
    *b"5%",
    *b"6^",
    *b"7&",
    *b"8*",
    *b"9(",
    *b"0)",
    *b"-_",
    *b"=+",
    *b"\0\0",//backspace
    *b"\0\0",//tab
    *b"qQ",
    *b"wW",
    *b"eE",
    *b"rR",
    *b"tT",
    *b"yY",
    *b"uU",
    *b"iI",
    *b"oO",
    *b"pP",
    *b"[{",
    *b"]}",
    *b"\n\n",
    *b"\0\0",//left_control
    *b"aA",
    *b"sS",
    *b"dD",
    *b"fF",
    *b"gG",
    *b"hH",
    *b"jJ",
    *b"kK",
    *b"lL",
    *b";:",
    *b"'\"",
    *b"`~",
    *b"\0\0",//left shift
    *b"\\|",
    *b"zZ",
    *b"xX",
    *b"cC",
    *b"vV",
    *b"bB",
    *b"nN",
    *b"mM",
    *b",<",
    *b".>",
    *b"/?",
    *b"\0\0",//right shift
    *b"**",
    *b"\0\0",//left alt
    *b"  ",
    *b"\0\0",//capslock
    ];

const TOUCH_RELEASE: u8 = 1 << 7;

fn check_key_state(key: u8) -> (bool, usize) {
    if (key & TOUCH_RELEASE) == TOUCH_RELEASE {
        (true, (key - TOUCH_RELEASE) as usize)
    } else {
        (false, key as usize)
    }
}

fn get_scancode() -> u8 {
    let mut scancode = 0;
    loop {
        if cpuio::inb(0x60) != scancode {
            scancode = cpuio::inb(0x60);
            if scancode > 0 {
                return scancode;
            }
        }
    }
}

pub fn kbd_callback() {
    static mut SHIFT: bool = false;
    static mut CTRL: bool = false;
    static mut ALT: bool = false;
    let scancode = get_scancode();
    let (is_release, scancode) = check_key_state(scancode);
    unsafe {
        //TODO remove unsafe
        match self::KEYMAP_US.get(scancode as usize) {
            Some(b"\0\0") => match scancode {
                0x2A | 0x36 => SHIFT = !is_release,
                0x38 => ALT = !is_release,
                0x1D => CTRL = !is_release,
                0x0E if !is_release => {
                    vga::VGA.backspace();
                }
                _ => {}
            },
            Some(ascii) if !is_release => {
                let sym = if SHIFT { ascii[1] } else { ascii[0] };
                vga::VGA.keypress(sym);
            }
            _ => {}
        }
    }
}
