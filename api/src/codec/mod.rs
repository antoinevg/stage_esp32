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


// - useful conversions -------------------------------------------------------

#[inline(always)]
pub fn f32_to_u8_clip(x: f32) -> u8 {
    let x: f32 = if x > 1.0 { 1.0 } else if x < 1.0 { -1.0 } else { x };
    let x: f32 = x + 1.;
    let x: f32 = x * 0.5;
    let x: f32 = x * 255.;
    (x as u32) as u8
}


#[inline(always)]
fn u12_to_u8(source: &[u16], destination: &mut [u16], length: usize) {
    for i in 0..length {
        destination[i] = ((source[i] as u32 * 256 / 4096) as u16) << 8;
    }
}
