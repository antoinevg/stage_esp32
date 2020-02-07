use esp_idf::{AsResult, EspError};

use crate::audio::{Config, OpaqueInterface};
use crate::logger;


// - modules ------------------------------------------------------------------

pub mod adac;
pub mod sgtl5000;


// - global constants ---------------------------------------------------------

const TAG: &str = "api::codec::mod";


// - types --------------------------------------------------------------------

pub unsafe trait Codec {
    fn new() -> Self;

    fn start_c(&self, config: &Config,
               opaque_interface_ptr: *const OpaqueInterface) -> Result<(), EspError>;

    fn init(&mut self, config: &Config) -> Result<(), EspError>;

    fn read(&self, config: &Config, callback_buffer: &mut [f32]) -> Result<(), EspError>;
    fn write(&self, config: &Config, callback_buffer: &[f32]) -> Result<(), EspError>;

    fn test(&self) -> () {
        log!(TAG, "Codec::test");
    }
}
