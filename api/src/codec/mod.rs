use esp_idf::{AsResult, EspError};

use crate::audio;
use crate::logger;


// - modules ------------------------------------------------------------------

pub mod adac;
pub mod sgtl5000;


// - global constants ---------------------------------------------------------

const TAG: &str = "api::codec::mod";


// - types --------------------------------------------------------------------

pub trait Codec {
    fn new() -> Self;

    fn init (&self, config: &audio::Config) -> Result<(), EspError>;
    fn start(&self, config: &audio::Config) -> Result<(), EspError>;

    fn start_c(&self, config: &audio::Config,
               opaque_interface_ptr: *const audio::OpaqueInterface) -> Result<(), EspError>;

    fn test(&self) -> () {
        log!(TAG, "Codec::test");
    }
}
