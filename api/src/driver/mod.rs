use esp_idf::{AsResult, EspError};

use crate::audio;
use crate::display;
use crate::logger;


// - modules ------------------------------------------------------------------

pub mod adac;
pub mod sgtl5000;
pub mod sh1106;
pub mod wm8731;


// - global constants ---------------------------------------------------------

const TAG: &str = "api::driver";


// - types --------------------------------------------------------------------

pub unsafe trait Codec {
    fn new() -> Self;

    fn start_c(&self, config: &audio::Config,
               opaque_interface_ptr: *const audio::OpaqueInterface) -> Result<(), EspError>;

    fn init(&mut self, config: &audio::Config) -> Result<(), EspError>;

    fn read(&self, config: &audio::Config, callback_buffer: &mut [f32]) -> Result<(), EspError>;
    fn write(&self, config: &audio::Config, callback_buffer: &[f32]) -> Result<(), EspError>;
}


pub unsafe trait Display {
    fn new() -> Self;

    fn init(&mut self, config: &display::Config) -> Result<(), EspError>;

    fn write(&self) -> Result<(), EspError>;
}
