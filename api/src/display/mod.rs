extern crate alloc;
use alloc::boxed::Box;

use cty::{c_int, c_void};

use esp_idf::bindings as idf;
use esp_idf::{AsResult, EspError, portMAX_DELAY, portMUX_INITIALIZER_UNLOCKED};

use crate::driver;
use crate::logger;


// - global constants ---------------------------------------------------------

const TAG: &str = "api::display";


// - display::Interface -------------------------------------------------------

#[repr(C)]
pub struct Config {
    pub width: usize,
    pub height: usize,
}


pub struct Interface<D> {
    pub config: Config,
    pub driver: D,
}


impl<D> Interface<D>
where D: driver::Display {
    pub fn new(width: usize, height: usize) -> Interface<D> {
        Interface {
            config: Config {
                width: width,
                height: height,
            },
            driver: D::new(),
        }
    }

    pub fn start(&mut self) -> Result<(), EspError> {
        self.driver.init(&mut self.config)
    }

    pub fn write(&self, frame_buffer: &[u8]) -> Result<(), EspError> {
        self.driver.write(frame_buffer)
    }
}


// - embedded_graphics --------------------------------------------------------

// TODO move this to driver::sh1106 - chances are we'll have to do one
// of these for each physical display we want to support

const WIDTH: usize = 128;
const HEIGHT: usize = 64;
const PAGES: usize = 8;

use embedded_graphics::{
    drawable::Pixel,
    pixelcolor::PixelColor,
    Drawing
};

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

pub struct Display {
    pub frame_buffer: [u8; WIDTH * PAGES],
}

impl Display {
    pub fn new() -> Display {
        Display {
            frame_buffer: [0; WIDTH * PAGES],
        }
    }
}

impl Drawing<CustomPixelColor> for Display {
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
