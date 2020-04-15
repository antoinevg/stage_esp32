extern crate alloc;
use alloc::boxed::Box;

use cty::{c_int, c_void};

use esp_idf::bindings as idf;
use esp_idf::{AsResult, EspError, portMAX_DELAY, portMUX_INITIALIZER_UNLOCKED};

use crate::driver;
use crate::logger;


// - global constants ---------------------------------------------------------

const TAG: &str = "api::display";


// - types --------------------------------------------------------------------

pub type Buffer = [u8];


// - display::Interface -------------------------------------------------------

#[repr(C)]
pub struct Config {
    pub width: usize,
    pub height: usize,
}


pub struct Interface<'a, D> {
    pub config: Config,
    pub driver: D,
    pub closure: Box<dyn FnMut(&mut Buffer) + 'a>,
    pub frame_buffer: &'a mut [u8],
}


impl<'a, D> Interface<'a, D>
where D: driver::Display {
    pub fn new<F: FnMut(&mut Buffer) + 'a>(width: usize, height: usize, frame_buffer: &'a mut [u8], closure: F) -> Interface<'a, D> {
        Interface {
            config: Config {
                width: width,
                height: height,
            },
            driver: D::new(),
            closure: Box::new(closure),
            frame_buffer: frame_buffer,
        }
    }

    pub fn start(&mut self) -> Result<(), EspError> {
        self.driver.init(&mut self.config)
    }

    pub fn refresh(&mut self) -> Result<(), EspError> {
        (self.closure)(self.frame_buffer);
        Ok(())
    }

    pub fn flush(&self) -> Result<(), EspError> {
        self.driver.write(self.frame_buffer)
    }
}


// - embedded_graphics --------------------------------------------------------

// TODO move this to driver::sh1106 - chances are we'll have to do one
// of these for each physical display we want to support

const WIDTH: usize = 128; // TODO lose
const HEIGHT: usize = 64; // TODO lose
const PAGES: usize = 8;   // TODO lose

use embedded_graphics::{
    drawable::Pixel,
    pixelcolor::PixelColor,
    Drawing
};


pub struct Display <'a> {
    pub frame_buffer: &'a mut [u8],
}

impl<'a> Display<'a> {
    pub fn new(frame_buffer: &'a mut [u8]) -> Display {
        Display {
            frame_buffer: frame_buffer,
        }
    }

    pub fn clear(&mut self) -> () {
        for pixel in self.frame_buffer.iter_mut() {
            *pixel = 0;
        }
    }
}

impl Drawing<CustomPixelColor> for Display<'_> {
    fn draw<T>(&mut self, item_pixels: T) where T: IntoIterator<Item = Pixel<CustomPixelColor>> {
        for Pixel(coord, color) in item_pixels {
            let x = coord[0] as usize;
            let y = coord[1] as usize;
            if x >= WIDTH || y >= HEIGHT {
                continue;
            }

            let page = y / 8;
            if page >= PAGES {
                continue;
            }

            let address = (page * WIDTH) + x;
            if color.value == 1 {
                self.frame_buffer[address] |= 1 << (y % 8);
            } else {
                self.frame_buffer[address] &= !(1 << (y % 8));
            }
        }
    }
}


#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CustomPixelColor {
    pub value: u8,
}

impl CustomPixelColor {
    fn new(color: u8) -> Self {
        CustomPixelColor { value: color }
    }
}

impl PixelColor for CustomPixelColor {}

impl From<u8> for CustomPixelColor {
    fn from(other: u8) -> Self {
        CustomPixelColor {
            value: other,
        }
    }
}
