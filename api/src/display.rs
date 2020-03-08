extern crate alloc;
use alloc::boxed::Box;

use cty::{c_int, c_void};

use esp_idf::bindings as idf;
use esp_idf::{AsResult, EspError, portMAX_DELAY, portMUX_INITIALIZER_UNLOCKED};

use crate::logger;

use crate::driver;


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

        // initialize driver
        self.driver.init(&mut self.config)?;

        Ok(())
    }
}
