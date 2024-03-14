use volatile::Volatile;
use core::fmt::{self, write, Write};
use lazy_static::lazy_static;
use spin::Mutex;

#[cfg(test)]
use crate::{serial_print, serial_println};


#[derive(Clone, Debug, Eq, PartialEq, Copy)]
#[repr(C)]
#[allow(dead_code)]
pub enum Color {
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

#[derive(Clone, Debug, Eq, PartialEq, Copy)]
#[repr(transparent)]
pub struct UnitColor {
    unit_color : u8
}

impl UnitColor {
    fn new(forecolor : Color, backgroundcolor : Color) -> UnitColor {
        UnitColor{unit_color :(forecolor as u8) << 4 | (backgroundcolor as u8)}
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Copy)]
#[repr(C)]
struct Unit {
    ascii_code : u8,
    unit_color : UnitColor
}


const VGA_BUFFER_WIDTH : usize = 80;
const VGA_BUFFER_HEIGHT : usize = 25;

#[repr(transparent)]
struct VgaBuffer {
    chars : [[Volatile<Unit>; VGA_BUFFER_WIDTH];VGA_BUFFER_HEIGHT]
}

pub struct Writer {
    vga_buffer : &'static mut VgaBuffer,
    cursor_x : u8,
    cursor_y : u8,
    unit_color : UnitColor
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s : &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.new_line();
            }
            byte => {
                if self.cursor_y >= VGA_BUFFER_WIDTH as u8 {
                    self.new_line();
                }

                let color_code = self.unit_color;
                let raw = self.cursor_x as usize;
                let col = self.cursor_y as usize;
                self.vga_buffer.chars[raw][col].write(Unit {
                    ascii_code : byte,
                    unit_color : color_code
                });
                self.cursor_y +=1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }
    
    pub fn new_line(&mut self) {
        let max_height = VGA_BUFFER_HEIGHT as u8;
        if self.cursor_x == 24 {
            for raw in 1..VGA_BUFFER_HEIGHT {
                for col in 0..VGA_BUFFER_WIDTH {
                    let c = self.vga_buffer.chars[raw][col].read();
                    self.vga_buffer.chars[raw-1][col].write(c);
                }
            }
            self.clear_raw();
        } else {
            self.cursor_x += 1;
            self.cursor_y = 0;
        }
        
    }

    pub fn clear_raw(&mut self) {
        let x = self.cursor_x as usize;
        let blank = Unit {
            ascii_code : b' ',
            unit_color : UnitColor::new(Color::Black, Color::Black)
        };
        for i in 0..VGA_BUFFER_WIDTH {
            self.vga_buffer.chars[x][i].write(blank);
        }
        self.cursor_y = 0;
    }
}

lazy_static! {
    pub static ref WRITER : Mutex<Writer> = Mutex::new(Writer {
        vga_buffer : unsafe { &mut *(0xb8000 as *mut VgaBuffer) },
        cursor_x : 0,
        cursor_y : 0,
        unit_color : UnitColor::new(Color::Yellow, Color::Black)
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {

    
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n",format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args : fmt::Arguments) {
    // use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[test_case]
fn test_println_many() {
    serial_print!("test_println_many... ");
    for _ in 0..200 {
        println!("test_println_many output");
    }
    serial_println!("[ok]");
}

#[test_case]
fn test_println_output() {
    serial_print!("test_println_output... ");

    let s = "Some test string that fits on a single line";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().vga_buffer.chars[VGA_BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_code), c);
    }

    serial_println!("[ok]");
}

// pub fn print_something() {
//     let mut writer = Writer {
//         cursor_x: 0,
//         cursor_y: 0,
//         unit_color: UnitColor::new(Color::Yellow, Color::Black),
//         vga_buffer: unsafe { &mut *(0xb8000 as *mut VgaBuffer) },
//     };

//     writer.write_byte(b'H');
//     writer.write_byte(b' ');
//     writer.write_byte(b'Y');
//     writer.write_byte(b'\n');
//     writer.write_byte(b'Z');

//     writer.write_string("\nHello ");
//     writer.write_string("Wörld!\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\nasdasdasdasdffffffffffffffffffffffssssssssssssssssssssssfffffxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxdgfsgreggggggggggggggggggggggggggggg");
// }