use core::fmt;
///////////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // forces u8 numbers
pub enum Colour {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

///////////////////////////////////////////////////////////////////////////////

// new colour data format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColourCode(u8);

/// ColourCode is a new type that contains both foreground and background Colours
/// for easy use in VGA text calling

impl ColourCode {
    fn new(foreground: Colour, background: Colour) -> ColourCode {
        ColourCode((background as u8) << 4 | (foreground as u8))
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // work just like C
struct ScreenChar { // creates a struct that contains both character and colour data
    ascii_character: u8,
    colour_code: ColourCode,
}

// defines the size of the screen screen i think? need to check this later.
pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

use volatile::Volatile;

#[repr(transparent)]
struct Buffer { // holds all possible selections, with Colour and characters. 
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

///////////////////////////////////////////////////////////////////////////////

pub struct Writer { // a writing structure with Colour codes, positions of the char, and the buffer that holds all chars
    column_position: usize,
    colour_code: ColourCode,
    buffer: &'static mut Buffer,
}

///////////////////////////////////////////////////////////////////////////////

impl Writer { // writes one byte
    pub fn new_line(&mut self) { // new line handler
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    pub fn clear_row(&mut self, row: usize) { // rewrites a whole row with empty
        let blank = ScreenChar {
            ascii_character: b' ',
            colour_code: self.colour_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(), // handles the new line character
            byte => { // if it is anything else
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let colour_code = self.colour_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    colour_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) { // loops over the string, writes it out in bytes
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        colour_code: ColourCode::new(Colour::Yellow, Colour::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

///////////////////////////////////////////////////////////////////////////////
/// macro bullshit

impl fmt::Write for Writer { // implements the write_fmt trait for Writer
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;   // new

    interrupts::without_interrupts(|| {     // new
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! print { // similar to the print! macro but to VGA
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println { // print! macro but with a tailing \n
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

///////////////////////////////////////////////////////////////////////////////
/// ChatGPT wrote this don't come to me if it breaks
#[allow(dead_code)]
const MAX_SIZE: usize = 1024;

#[allow(dead_code)]
struct String {
    data: [u8; MAX_SIZE],
    len: usize,
}

impl String {
    #[allow(dead_code)]
    fn new() -> Self {
        String { data: [0; MAX_SIZE], len: 0 }
    }

    #[allow(dead_code)]
    fn push(&mut self, c: u8) {
        if self.len == MAX_SIZE {
            panic!("string is full");
        }
        self.data[self.len] = c;
        self.len += 1;
    }

    #[allow(dead_code)]
    fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }
}

// Made the String type lmao
///////////////////////////////////////////////////////////////////////////////
/*
pub fn print(output: &str) {
    let mut writer = Writer {
        column_position: 0,
        Colour_code: ColourCode::new(Colour::Yellow, Colour::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };
    writer.write_string(output);
}

pub fn println(output: &str) {
    print(output);
    print("\n")
}

pub fn print_special(output: &str, foreground: Colour, background: Colour) {
    let mut writer = Writer {
        column_position: 0,
        Colour_code: ColourCode::new(foreground, background),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_string(output);
}

pub fn println_special(output: &str, foreground: Colour, background: Colour) {
    print_special(output, foreground, background);
    print_special("\n", foreground, background);
}
*/

///////////////////////////////////////////////////////////////////////////////

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}

#[test_case]
fn trivial_assertion() {
    print!("trivial assertion...\t");
    assert_eq!(1, 1);
    println!(" [ok]")
}

#[test_case]
fn test_println_many() {
    for _ in 0..3 {
        println!("println multiple simple output");
    }
}
