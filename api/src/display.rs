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
    pub resolution: (u32, u32),
}


pub struct Interface<D> {
    pub config: Config,
    pub driver: D,
}


impl<D> Interface<D>
where D: driver::Display {
    pub fn new(resolution: (u32, u32)) -> Interface<D> {
        Interface {
            config: Config {
                resolution: resolution,
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
