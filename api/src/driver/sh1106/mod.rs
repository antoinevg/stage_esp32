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

pub const CONFIG: Config = Config {
    width: 128,
    height: 64,
    pages: 8,
};


// - driver -------------------------------------------------------------------

pub struct Driver {
    pub reset_pin: idf::gpio_num_t, // not used in current revision
    pub i2c_pins: crate::i2c::Pins,
    pub spi_pins: spi::Pins,
    spi_handle: Option<esp_idf::bindings::spi_device_handle_t>,
}


unsafe impl Display for Driver {
    fn new() -> Driver {
        Driver {
            reset_pin: idf::gpio_num_t::GPIO_NUM_23,
            i2c_pins: crate::i2c::Pins::new(),
            spi_pins: spi::Pins::new(),
            spi_handle: None,
        }
    }

    fn init(&mut self, config: &Config) -> Result<(), EspError> {
        let display_address = 0x3c;
        let i2c_port = idf::i2c_port_t::I2C_NUM_0;
        let spi_device = idf::spi_host_device_t::SPI3_HOST;

        log!(TAG, "initialize display subsystem");

        // - i2c ------------------------------
        // initialize i2c peripheral
        //log!(TAG, "initialize i2c display peripheral");
        //unsafe { i2c::init(i2c_port, self.i2c_pins)?; }
        // configure display over i2c
        //log!(TAG, "configure display over i2c");
        //unsafe { i2c::configure(self.reset_pin, i2c_port, display_address)?; }

        // - spi ------------------------------
        log!(TAG, "initialize spi display peripheral");
        self.spi_handle = Some(unsafe { spi::init(spi_device, self.spi_pins)? });
        unsafe { spi::configure(self.spi_handle.unwrap(), self.spi_pins.dc)?; }

        Ok(())
    }

    fn write(&self, frame_buffer: &[u8]) -> Result<(), EspError> {
        let display_address = 0x3c;
        let gpio_dc = self.spi_pins.dc;
        let command = |bytes: &[u8]| -> Result<(), EspError> {
            unsafe { spi::transmit(self.spi_handle.unwrap(), gpio_dc, bytes, spi::Mode::Command) }
        };

        for page in 0usize..CONFIG.pages {
            let page_address = (0xb0 + page) as u8;
            command(&[page_address])?;                         // set page address
            command(&[spi::Register::SETLOWCOLUMN.into()])?;   // set lower column address
            command(&[spi::Register::SETHIGHCOLUMN.into()])?;  // set higher column address
            command(&[spi::Register::SETSTARTLINE.into()])?;

            let page_start = page * CONFIG.width;
            let page_end = page_start + CONFIG.width;
            let page_buffer = &frame_buffer[page_start..page_end];
            unsafe { spi::transmit(self.spi_handle.unwrap(), gpio_dc, &page_buffer, spi::Mode::Data)?; }
        }

        Ok(())
    }
}
