use esp_idf::bindings as idf;
use esp_idf::{AsResult, EspError};

use crate::audio;
use crate::logger;

// - modules ------------------------------------------------------------------

pub mod adac;
pub mod sgtl5000;


// - global constants ---------------------------------------------------------

const TAG: &str = "api::codec::mod";

pub const FOO: u32 = 23;


// - types --------------------------------------------------------------------

pub trait Driver {
    fn new() -> Self;

    fn init (&self, config: &audio::Config) -> Result<(), EspError>;
    fn start(&self, config: &audio::Config) -> Result<(), EspError>;

    fn start_c(&self, config: &audio::Config,
               opaque_interface_ptr: *const audio::OpaqueInterface) -> Result<(), EspError>;

    fn test(&self) -> () {
        log!(TAG, "Driver::test");
    }
}


// ----------------------------------------------------------------------------

pub struct SGTL5000 {
}


impl Driver for SGTL5000 {
    fn new() -> SGTL5000 {
        SGTL5000 { }
    }

    fn init(&self, config: &audio::Config) -> Result<(), EspError> {
        // TODO
        Ok(())
    }

    fn start(&self, config: &audio::Config) -> Result<(), EspError> {
        // TODO
        Ok(())
    }

    fn start_c(&self, config: &audio::Config,
               opaque_interface_ptr: *const audio::OpaqueInterface) -> Result<(), EspError> {
        log!(TAG, "SGTL5000::(Driver)::start");
        unsafe {
            audio::C_codec_sgtl5000_start(opaque_interface_ptr,
                                          config.fs,
                                          config.num_channels,
                                          config.word_size,
                                          config.block_size).as_result()?;
        }

        Ok(())
    }
}


// ----------------------------------------------------------------------------

pub struct ADAC {
}


impl Driver for ADAC {
    fn new() -> ADAC {
        ADAC { }
    }

    fn init(&self, config: &audio::Config) -> Result<(), EspError> {
        // TODO
        Ok(())
    }

    fn start(&self, config: &audio::Config) -> Result<(), EspError> {
        // TODO
        Ok(())
    }

    fn start_c(&self, config: &audio::Config,
               opaque_interface_ptr: *const audio::OpaqueInterface) -> Result<(), EspError> {
        log!(TAG, "ADAC::(Driver)::start");
        unsafe {
            audio::C_codec_adac_start(opaque_interface_ptr,
                                      config.fs,
                                      config.num_channels,
                                      config.word_size,
                                      config.block_size).as_result()?;
        }

        Ok(())
    }
}
