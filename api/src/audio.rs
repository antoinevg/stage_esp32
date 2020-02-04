extern crate alloc;
use alloc::boxed::Box;

use cty::{c_void, c_uchar, c_float};

use esp_idf::bindings as idf;
use esp_idf::{AsResult, EspError, portMAX_DELAY};

use crate::i2s;
use crate::logger;
use crate::wavetable;

use crate::codec;
use crate::codec::Driver;


// - global constants ---------------------------------------------------------

const TAG: &str = "api::audio";

pub const BLOCK_SIZE: usize = 128; // TODO get this from extern CONFIG_AUDIO_BLOCK_SIZE


// - types --------------------------------------------------------------------

pub type Buffer = [f32; BLOCK_SIZE];
type DmaBuffer = [u16; BLOCK_SIZE];


// - ffi types ----------------------------------------------------------------

#[repr(C)] pub struct OpaqueInterface { _private: [u8; 0] }


// - ffi imports --------------------------------------------------------------

extern "C" {
    pub fn C_codec_adac_start(opaque_interface_ptr: *const OpaqueInterface,
                              fs: f32,
                              num_channels: usize,
                              word_size: usize,
                              block_size: usize) -> idf::esp_err_t;

    pub fn C_codec_sgtl5000_start(opaque_interface_ptr: *const OpaqueInterface,
                                  fs: f32,
                                  num_channels: usize,
                                  word_size: usize,
                                  block_size: usize) -> idf::esp_err_t;
}


// - ffi exports --------------------------------------------------------------

#[no_mangle]
extern "C" fn RUST_codec_adac_callback(opaque_interface_ptr: *const OpaqueInterface,
                                       fs: f32,
                                       num_channels: usize,
                                       buffer_ptr: *mut c_float,
                                       buffer_size: usize) {
    if buffer_size != BLOCK_SIZE {
        panic!("audio::rust_audio_interface_callback callback buffer size does not match BLOCK_SIZE");
    }

    unsafe {
        let buffer = core::mem::transmute::<*mut c_float, &mut Buffer>(buffer_ptr);
        let interface_ptr = core::mem::transmute::<*const OpaqueInterface,
                                                   *mut Interface<codec::ADAC>>(opaque_interface_ptr);

        ((*interface_ptr).closure)(fs, num_channels, buffer);
    }
}


#[no_mangle]
extern "C" fn RUST_codec_sgtl5000_callback(opaque_interface_ptr: *const OpaqueInterface,
                                           fs: f32,
                                           num_channels: usize,
                                           buffer_ptr: *mut c_float,
                                           buffer_size: usize) {
    if buffer_size != BLOCK_SIZE {
        panic!("audio::rust_audio_interface_callback callback buffer size does not match BLOCK_SIZE");
    }

    unsafe {
        let buffer = core::mem::transmute::<*mut c_float, &mut Buffer>(buffer_ptr);
        let interface_ptr = core::mem::transmute::<*const OpaqueInterface,
                                                   *mut Interface<codec::SGTL5000>>(opaque_interface_ptr);

        ((*interface_ptr).closure)(fs, num_channels, buffer);
    }
}


// - audio::Interface ---------------------------------------------------------

#[repr(C)]
pub struct Config {
    pub fs: f32,
    pub num_channels: usize,
    pub word_size: usize,
    pub block_size: usize,
}


pub struct Interface<'a, D> {
    config: Config,
    driver: D,
    closure: Box<dyn FnMut(f32, usize, &mut Buffer) + 'a>,
}


impl<'a, D> Interface<'a, D>
where D: codec::Driver {
    pub fn new<F: FnMut(f32, usize, &mut Buffer) + 'a>(fs: f32, closure: F) -> Interface<'a, D> {
        Interface {
            config: Config {
                fs: fs,
                num_channels: 2,
                word_size: 2,
                block_size: BLOCK_SIZE,
            },
            driver: D::new(),
            closure: Box::new(closure),
        }
    }

    pub fn start_c(&self) -> Result<(), EspError> {
        let opaque_interface_ptr = unsafe {
            core::mem::transmute::<*const Interface<D>,
                                   *const OpaqueInterface>(self)
        };

        self.driver.init(&self.config)?;
        self.driver.start_c(&self.config, opaque_interface_ptr)?;
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), EspError> {
        init_pure_rust(self.config.fs,
                       self.config.num_channels,
                       self.config.word_size,
                       self.config.block_size,
                       |fs, num_channels, buffer| {
                           // TODO use self.config.callback instead
                       })
    }

}


// - test callbacks -----------------------------------------------------------

struct TestState {
    channel_1: (f32, f32),  // phase, sample
    channel_2: (f32, f32),
}

static mut TEST_STATE: TestState = TestState {
    channel_1: (0., 0.),
    channel_2: (0., 0.),
};

pub fn test_callback(fs: f32, num_channels: usize, buffer: &mut Buffer) -> () {
    let num_frames = buffer.len() / num_channels;
    for f in 0..num_frames {
        let x = f * num_channels;
        unsafe {
            TEST_STATE.channel_1 = test_signal_sin(fs, 110., TEST_STATE.channel_1.0);
            TEST_STATE.channel_2 = test_signal_saw(fs, 110., TEST_STATE.channel_2.0);
            buffer[x+0] = TEST_STATE.channel_2.1; // right
            buffer[x+1] = TEST_STATE.channel_1.1; // left
        }
    }
}


// - test signals -------------------------------------------------------------

const PI:  f32 = 3.14159265358979323846264338327950288_f32; // π
const TAU: f32 = 6.28318530717958647692528676655900576_f32; // 2π

pub fn test_signal_one(fs: f32, f: f32, phase: f32) -> (f32, f32) {
    let dx = f / fs;
    let w = 2. * PI * dx;

    let sample = unsafe { idf::cosf(phase * w) };

    let phase = phase + 1.;
    //let phase = if phase > PI { phase - TAU } else { phase };

    return (phase, sample);
}


pub fn test_signal_two(fs: f32, f: f32, phase: f32) -> (f32, f32) {
    let sample = phase / PI;

    let dx = f / fs;
    let phase = phase + dx;
    let phase = if phase > PI { phase - TAU } else { phase };

    return (phase, sample);
}


pub fn test_signal_sin(fs: f32, f: f32, phase: f32) -> (f32, f32) {
    let wt_index = phase * (wavetable::LENGTH - 1) as f32;
    let sample = wavetable::SIN[wt_index as usize];

    let dx = f / fs;
    let phase = phase + dx;
    let phase = if phase > 1.0 { phase - 1.0 } else { phase };

    return (phase, sample);
}


pub fn test_signal_saw(fs: f32, f: f32, phase: f32) -> (f32, f32) {
    let wt_index = phase * (wavetable::LENGTH - 1) as f32;
    let sample = wavetable::SAW[wt_index as usize];

    let dx = f / fs;
    let phase = phase + dx;
    let phase = if phase > 1.0 { phase - 1.0 } else { phase };

    return (phase, sample);
}


// - conversions --------------------------------------------------------------

fn clip_convert_u8(x: f32) -> u8 {
    let x: f32 = if x > 1.0 { 1.0 } else if x < 1.0 { -1.0 } else { x };
    let x: f32 = x + 1.;
    let x: f32 = x / 2.;
    let x: f32 = x * 255.;
    (x as u32) as u8
}


fn u12_to_u8(source: &DmaBuffer, destination: &mut DmaBuffer, length: usize) {
    for i in 0..length {
        destination[i] = ((source[i] as u32 * 256 / 4096) as u16) << 8;
    }
}


// - pure rust implementation -------------------------------------------------

type Callback = fn(fs: f32, num_channels: usize, buffer: &mut Buffer) -> ();

static mut AUDIOCONFIG: Config = Config {
    fs: 48000.,
    num_channels: 2,
    word_size: 2,
    block_size: BLOCK_SIZE,
};


pub fn init_pure_rust(fs: f32, num_channels: usize, word_size: usize, block_size: usize, callback: Callback) -> Result<(), EspError> {
    log!(TAG, "initializing audio subsystem");

    log!(TAG, "initializing i2s driver with fs: {}", fs);
    unsafe { i2s::init(fs, block_size) }?;

    log!(TAG, "starting audio::__esp32_wrap_audio_task__");

    let stack_depth = 8192;
    let priority = 5;
    let core_id = 1; // idf::tskNO_AFFINITY as i32;
    unsafe {
        let mut task_handle = core::mem::zeroed::<c_void>();

        let task_handle_ptr = &mut task_handle as *mut _ as *mut idf::TaskHandle_t;
        let audioconfig_ptr = &mut AUDIOCONFIG as *mut _ as *mut c_void;

        //idf::xTaskCreatePinnedToCore(Some(esp32_wrap_audio_task_passthrough),
        idf::xTaskCreatePinnedToCore(Some(esp32_wrap_audio_task_testsignal),
                                     "audio::thread".as_bytes().as_ptr() as *const i8,
                                     stack_depth,
                                     audioconfig_ptr,
                                     priority,
                                     task_handle_ptr,
                                     core_id);
    }

    Ok(())
}


#[allow(unused_assignments)]
#[no_mangle]
pub unsafe extern "C" fn esp32_wrap_audio_task_testsignal(config_ptr: *mut c_void) {
    let config = core::mem::transmute::<*mut c_void, &mut Config>(config_ptr);

    let mut write_buffer: DmaBuffer = [0; BLOCK_SIZE];
    let mut channel_1 = (0., 0.); // phase, sample
    let mut channel_2 = (0., 0.);

    let buffer_size = BLOCK_SIZE * config.word_size;
    let num_frames = BLOCK_SIZE / config.num_channels;

    loop {
        for t in 0..num_frames {
            let x = t * 2;
            channel_1 = test_signal_sin(config.fs, 110., channel_1.0);
            channel_2 = test_signal_saw(config.fs, 110., channel_2.0);
            let sample_1 = (clip_convert_u8(channel_1.1) as u16) << 8;
            let sample_2 = (clip_convert_u8(channel_2.1) as u16) << 8;
            write_buffer[x+1] = sample_1;
            write_buffer[x]   = sample_2;
        }
        let mut bytes_written = 0;
        idf::i2s_write(i2s::PORT,
                       write_buffer.as_ptr() as *const core::ffi::c_void,
                       buffer_size,
                       &mut bytes_written,
                       portMAX_DELAY);
    }
}


#[allow(unused_assignments)]
#[no_mangle]
unsafe extern "C" fn esp32_wrap_audio_task_passthrough(config_ptr: *mut c_void) {
    let config = core::mem::transmute::<*mut c_void, &mut Config>(config_ptr);

    let mut read_buffer: DmaBuffer = [0; BLOCK_SIZE];
    let mut write_buffer: DmaBuffer = [0; BLOCK_SIZE];

    let buffer_size = BLOCK_SIZE * config.word_size;

    loop {
        let mut bytes_read = 0;
        idf::i2s_read(i2s::PORT,
                      read_buffer.as_mut_ptr() as *mut core::ffi::c_void,
                      buffer_size,
                      &mut bytes_read,
                      portMAX_DELAY);

        u12_to_u8(&read_buffer, &mut write_buffer, BLOCK_SIZE);

        let mut bytes_written = 0;
        idf::i2s_write(i2s::PORT,
                       write_buffer.as_ptr() as *const core::ffi::c_void,
                       buffer_size,
                       &mut bytes_written,
                       portMAX_DELAY);
    }
}
