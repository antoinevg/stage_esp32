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

// TODO move to driver
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Config {
    pub width: usize,
    pub height: usize,
    pub pages: usize,
}

pub struct Interface<'a, D> {
    pub config: Config,
    pub driver: D,
    pub closure: Box<dyn FnMut(Config, &mut Buffer) + 'a>,
    pub frame_buffer: &'a mut [u8],
}


impl<'a, D> Interface<'a, D>
where D: driver::Display {
    pub fn new<F: FnMut(Config, &mut Buffer) + 'a>(config: Config, frame_buffer: &'a mut [u8], closure: F) -> Interface<'a, D> {
        Interface {
            config: config,
            driver: D::new(),
            closure: Box::new(closure),
            frame_buffer: frame_buffer,
        }
    }

    pub fn start(&mut self) -> Result<(), EspError> {
        self.driver.init(&mut self.config)
    }

    pub fn refresh(&mut self) -> Result<(), EspError> {
        (self.closure)(self.config, self.frame_buffer);
        Ok(())
    }

    pub fn flush(&self) -> Result<(), EspError> {
        self.driver.write(self.frame_buffer)
    }
}


// - embedded_graphics --------------------------------------------------------

// TODO move this to driver::sh1106 - chances are we'll have to do one
// of these for each physical display we want to support

use embedded_graphics::{
    drawable::Pixel,
    pixelcolor::PixelColor,
    Drawing
};

pub struct Display <'a> {
    pub config: Config, // TODO move to driver
    pub frame_buffer: &'a mut [u8],
}

impl<'a> Display<'a> {
    pub fn new(config: Config, frame_buffer: &'a mut [u8]) -> Display {
        Display {
            config: config,
            frame_buffer: frame_buffer,
        }
    }

    pub fn clear(&mut self) -> () {
        for pixel in self.frame_buffer.iter_mut() {
            *pixel = 0;
        }
    }
}

// TODO move to driver or use driver
impl Drawing<CustomPixelColor> for Display<'_> {
    fn draw<T>(&mut self, item_pixels: T) where T: IntoIterator<Item = Pixel<CustomPixelColor>> {
        for Pixel(coord, color) in item_pixels {
            let x = coord[0] as usize;
            let y = coord[1] as usize;
            if x >= self.config.width || y >= self.config.height {
                continue;
            }

            let page = y / 8;
            if page >= self.config.pages {
                continue;
            }

            let address = (page * self.config.width) + x;
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
