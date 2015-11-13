use core::ptr::Unique;
use spin::Mutex;

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        $crate::vga_buffer::WRITER.lock().write_fmt(format_args!($($arg)*)).unwrap();
    });
}

#[allow(dead_code)]
#[repr(u8)]
pub enum Color {
    Black       = 0,
    Blue        = 1,
    Green       = 2,
    Cyan        = 3,
    Red         = 4,
    Magenta     = 5,
    Brown       = 6,
    LightGray   = 7,
    DarkGray    = 8,
    LightBlue   = 9,
    LightGreen  = 10,
    LightCyan   = 11,
    LightRed    = 12,
    Pink        = 13,
    Yellow      = 14,
    White       = 15,
}

#[derive(Clone, Copy)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

pub static WRITER: Mutex<Writer> = Mutex::new(Writer {
    column_position: 0,
    row_position: 0,
    color_code: ColorCode::new(Color::LightGreen, Color::Black),
    buffer: unsafe {Unique::new(0xb8000 as *mut _)},
});

struct Buffer {
    chars: [ScreenChar; BUFFER_WIDTH * BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    buffer: Unique<Buffer>,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                let row = self.row_position;
                let col = self.column_position;
                
                self.buffer().chars[row * BUFFER_WIDTH + col] = ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                };
                self.column_position += 1;
            }
        }

        self.scroll();
    }
    
    fn buffer(&mut self) -> &mut Buffer {
        unsafe { self.buffer.get_mut() }
    }

    fn scroll(&mut self) {
        if self.row_position > BUFFER_HEIGHT - 1 {
            let blank = ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            };
            {
                let buffer = self.buffer();
                for i in 0..((BUFFER_HEIGHT - 1) * (BUFFER_WIDTH)) {
                    buffer.chars[i] = buffer.chars[i + BUFFER_WIDTH];
                }

                for i in 
                    ((BUFFER_HEIGHT - 1) * (BUFFER_WIDTH))..
                    (BUFFER_HEIGHT * BUFFER_WIDTH) {

                    buffer.chars[i] = blank;
                }
            }

            self.row_position = BUFFER_HEIGHT - 1;
        }
    }
    
    fn new_line(&mut self) {
        self.column_position = 0;
        self.row_position += 1;
    }
 
    fn clear(&mut self) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for i in 0..(BUFFER_HEIGHT * BUFFER_WIDTH) {
            self.buffer().chars[i] = blank;
        }

    }

#[allow(dead_code)]
    fn clear_row(&mut self) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        let row = self.row_position;

        for i in (row * BUFFER_WIDTH)..(row * BUFFER_WIDTH + BUFFER_WIDTH) {
            self.buffer().chars[i] = blank;
        }
    }

#[allow(dead_code)]
	pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
        self.scroll();
    }
}

impl ::core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
        
        Ok(())
    }
}

pub fn clear_screen()
{
    WRITER.lock().clear();
}
