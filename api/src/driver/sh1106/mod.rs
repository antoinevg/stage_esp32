use esp_idf::bindings as idf;
use esp_idf::{AsResult, EspError, portMAX_DELAY};

use crate::display::{Config};
use crate::driver::Display;
use crate::logger;


// - modules ------------------------------------------------------------------

pub mod i2c;
pub mod spi;


// - global constants ---------------------------------------------------------

const TAG: &str = "api::driver::sh1106";


// - driver -------------------------------------------------------------------

pub struct Driver {
    pub i2c_pins: crate::i2c::Pins,
    pub spi_pins: spi::Pins,
    pub reset_pin: idf::gpio_num_t,
}


unsafe impl Display for Driver {
    fn new() -> Driver {
        Driver {
            i2c_pins: crate::i2c::Pins::new(),
            spi_pins: spi::Pins::new(),
            reset_pin: idf::gpio_num_t::GPIO_NUM_23,
        }
    }

    fn init(&mut self, config: &Config) -> Result<(), EspError> {
        let display_address = 0x3c;
        let i2c_port = idf::i2c_port_t::I2C_NUM_0;
        let spi_device = idf::spi_host_device_t::SPI3_HOST;

        log!(TAG, "initialize display subsystem");

        // TODO allocate memory etc.

        // - i2c ------------------------------
        // initialize i2c peripheral
        //log!(TAG, "initialize i2c display peripheral");
        //unsafe { i2c::init(i2c_port, self.i2c_pins)?; }
        // configure display over i2c
        //log!(TAG, "configure display over i2c");
        //unsafe { i2c::configure(self.reset_pin, i2c_port, display_address)?; }

        // - spi ------------------------------
        log!(TAG, "initialize spi display peripheral");
        let handle = unsafe { spi::init(spi_device, self.spi_pins)? };
        unsafe { spi::configure(self.spi_pins.dc, handle)?; }

        Ok(())
    }

    fn write(&self/*, config: &Config, callback_buffer: &[f32]*/) -> Result<(), EspError> {
        Ok(())
    }
}
