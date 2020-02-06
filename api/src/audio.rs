extern crate alloc;
use alloc::boxed::Box;

use cty::{c_int, c_void};

use esp_idf::bindings as idf;
use esp_idf::{AsResult, EspError, portMAX_DELAY, portMUX_INITIALIZER_UNLOCKED};

use crate::i2s;
use crate::logger;
use crate::wavetable;

use crate::codec::Codec;
use crate::codec::adac;
use crate::codec::sgtl5000;


// - global constants ---------------------------------------------------------

const TAG: &str = "api::audio";


// - types --------------------------------------------------------------------

pub type Buffer = [f32];


// - ffi types ----------------------------------------------------------------

#[repr(C)] pub struct OpaqueInterface { _private: [u8; 0] }


// - audio::Interface ---------------------------------------------------------

#[repr(C)]
pub struct Config {
    pub fs: f32,
    pub num_channels: usize,
    pub word_size: usize,
    pub block_size: usize,
}


pub struct Interface<'a, D> {
    pub config: Config,
    codec: D,
    pub closure: Box<dyn FnMut(f32, usize, &mut Buffer) + 'a>,
}


impl<'a, C> Interface<'a, C>
where C: Codec {
    pub fn new<F: FnMut(f32, usize, &mut Buffer) + 'a>(fs: f32, block_size: usize, closure: F) -> Interface<'a, C> {
        Interface {
            config: Config {
                fs: fs,
                num_channels: 2,
                word_size: 2,
                block_size: block_size,
            },
            codec: C::new(),
            closure: Box::new(closure),
        }
    }

    pub fn start_c(&self) -> Result<(), EspError> {
        let opaque_interface_ptr = unsafe {
            core::mem::transmute::<*const Interface<C>,
                                   *const OpaqueInterface>(self)
        };
        self.codec.start_c(&self.config, opaque_interface_ptr)
    }

    pub fn start(&mut self) -> Result<(), EspError> {
        let stack_depth = 8192;
        let priority = 5;
        let core_id = 1; // idf::tskNO_AFFINITY as i32;

        unsafe {
            #[no_mangle]
            extern "C" fn _Stage_api_audio_task_closure_wrapper(arg: *mut c_void) {
                let closure: &mut &mut dyn FnMut() -> () = unsafe { core::mem::transmute(arg) };
                closure()
            }

            let mut closure = || -> () {
                self.audio_thread();
            };

            let mut closure_ref: &mut dyn FnMut() -> () = &mut closure;
            let closure_rref = &mut closure_ref;
            let closure_ptr = closure_rref as *mut _ as *mut c_void;

            let mut task_handle = core::mem::zeroed::<c_void>();
            let task_handle_ptr = &mut task_handle as *mut _ as *mut idf::TaskHandle_t;

            self.codec.init(&self.config)?;

            idf::xTaskCreatePinnedToCore(Some(_Stage_api_audio_task_closure_wrapper),
                                         "audio::thread".as_bytes().as_ptr() as *const i8,
                                         stack_depth,
                                         closure_ptr,
                                         priority,
                                         task_handle_ptr,
                                         core_id);
        }
        Ok(())
    }

    unsafe fn audio_thread(&mut self) {
        const TAG: &str = "api::audio::thread";

        static mut MUX: idf::portMUX_TYPE = portMUX_INITIALIZER_UNLOCKED;

        let Config { fs, num_channels, word_size, block_size } = self.config;

        let buffer_size = block_size * word_size;
        let num_frames  = block_size / num_channels;
        let dma_read_buffer_ptr = idf::calloc(buffer_size as u32,
                                              core::mem::size_of::<u8>() as u32) as *mut u8;
        let dma_write_buffer_ptr = idf::calloc(buffer_size as u32,
                                               core::mem::size_of::<u8>() as u32) as *mut u8;
        let callback_buffer_ptr = idf::calloc(block_size as u32,
                                              core::mem::size_of::<f32>() as u32) as *mut f32;
        let mut state = State::new();

        log!(TAG, "starting audio with fs: {} blocksize: {}", fs, block_size);
        log!(TAG, "stack max: {}", idf::uxTaskGetStackHighWaterMark(core::ptr::null_mut()));

        loop {
            // TODO read data from i2s

            let dma_write_buffer: &mut [u8] = core::slice::from_raw_parts_mut(dma_write_buffer_ptr, buffer_size);
            let callback_buffer: &mut [f32] = core::slice::from_raw_parts_mut(callback_buffer_ptr, block_size);

            // invoke callback
            idf::vTaskEnterCritical(&mut MUX);
            /*for f in 0..num_frames {
                let x = f * self.config.num_channels;
                state.channel_1 = test_signal_sin(self.config.fs, 110., state.channel_1.0);
                state.channel_2 = test_signal_saw(self.config.fs, 110., state.channel_2.0);
                callback_buffer[x+0] = state.channel_2.1; // right
                callback_buffer[x+1] = state.channel_1.1; // left
            }*/
            //test_callback_inline(self.config.fs, self.config.num_channels, callback_buffer, &mut state);
            //test_callback(self.config.fs, self.config.num_channels, callback_buffer, &mut state);
            (self.closure)(self.config.fs, self.config.num_channels, callback_buffer);
            idf::vTaskExitCritical(&mut MUX);

            // convert callback_buffer data from f32 to u8
            for n in 0..num_frames {
                let index_f32 = n * self.config.num_channels;
                let right_f32 = callback_buffer[index_f32+0];
                let left_f32  = callback_buffer[index_f32+1];

                //let right_u8: u8 = clip_convert_u8(right_f32);
                //let left_u8: u8  = clip_convert_u8(left_f32);

                let right_f32 = if right_f32 > 1. { 1. } else if right_f32 < -1. { -1. } else { right_f32 };
                let left_f32  = if left_f32  > 1. { 1. } else if left_f32  < -1. { -1. } else { left_f32  };
                let right_u8: u8 = ((((right_f32 + 1.) * 0.5) * 255.0) as u32) as u8;
                let left_u8:  u8 = ((((left_f32  + 1.) * 0.5) * 255.0) as u32) as u8;

                let index_u8 = n * self.config.num_channels * self.config.word_size;
                dma_write_buffer[index_u8+0] = 0;
                dma_write_buffer[index_u8+1] = right_u8;
                dma_write_buffer[index_u8+2] = 0;
                dma_write_buffer[index_u8+3] = left_u8;
            }

            // write data to i2s
            let mut bytes_written = 0;
            idf::i2s_write(i2s::PORT,
                           dma_write_buffer.as_ptr() as *const core::ffi::c_void,
                           buffer_size,
                           &mut bytes_written,
                           portMAX_DELAY);
            if bytes_written != buffer_size {
                log!(TAG, "write mismatch buffer_size:{} != bytes_written:{}", buffer_size, bytes_written);
            }
        }
    }
}


// - test callbacks -----------------------------------------------------------

pub struct State {
    channel_1: (f32, f32),  // phase, sample
    channel_2: (f32, f32),
}

impl State {
    pub fn new() -> State {
        State {
            channel_1: (0., 0.),
            channel_2: (0., 0.),
        }
    }
}

fn test_callback(fs: f32, num_channels: usize, buffer: &mut Buffer, state: &mut State) {
    let num_frames = buffer.len() / num_channels;
    for f in 0..num_frames {
        let x = f * num_channels;

        state.channel_1 = test_signal_sin(fs, 110., state.channel_1.0);
        state.channel_2 = test_signal_saw(fs, 110., state.channel_2.0);

        buffer[x+0] = state.channel_2.1; // right
        buffer[x+1] = state.channel_1.1; // left
    }
}

#[inline(always)]
fn test_callback_inline(fs: f32, num_channels: usize, buffer: &mut Buffer, state: &mut State) {
    let num_frames = buffer.len() / num_channels;
    for f in 0..num_frames {
        let x = f * num_channels;

        state.channel_1 = test_signal_sin(fs, 110., state.channel_1.0);
        state.channel_2 = test_signal_saw(fs, 110., state.channel_2.0);

        buffer[x+0] = state.channel_2.1; // right
        buffer[x+1] = state.channel_1.1; // left
    }
}


// - test signals -------------------------------------------------------------

const PI:  f32 = 3.14159265358979323846264338327950288_f32; // π
const TAU: f32 = 6.28318530717958647692528676655900576_f32; // 2π


#[inline(always)]
fn interpolate_linear(wt: &[f32], index: f32) -> f32 {
    let wt_len = wt.len();
    let int_part: usize  = index as usize;
    let frac_part: f32 = index - int_part as f32;
    let x0 = int_part;
    let x1 = (x0 + 1) & (wt_len - 1);
    let y0 = wt[x0] as f32;
    let y1 = wt[x1] as f32;
    (y0 + ((y1 - y0) * frac_part))
}


#[inline(always)]
pub fn test_signal(fs: f32, f: f32, phase: f32) -> (f32, f32) {
    let sample = (phase * 2.) - 1.;

    let dx = f / fs;
    let phase = phase + dx;
    let phase = if phase > 1. { phase - 1. } else { phase };

    return (phase, sample);
}


#[inline(always)]
pub fn test_signal_cosf(fs: f32, f: f32, phase: f32) -> (f32, f32) {
    let dx = f / fs;
    let w = 2. * PI * dx;

    let sample = unsafe { idf::cosf(phase * w) };

    let phase = phase + 1.;

    return (phase, sample);
}


#[inline(always)]
pub fn test_signal_sin(fs: f32, f: f32, phase: f32) -> (f32, f32) {
    let wt_index = phase * (wavetable::LENGTH - 1) as f32;
    //let sample = wavetable::SIN[wt_index as usize];
    let sample = interpolate_linear(&wavetable::SIN, wt_index);

    let dx = f / fs;
    let phase = phase + dx;
    let phase = if phase > 1.0 { phase - 1.0 } else { phase };

    return (phase, sample);
}


#[inline(always)]
pub fn test_signal_saw(fs: f32, f: f32, phase: f32) -> (f32, f32) {
    let wt_index = phase * (wavetable::LENGTH - 1) as f32;
    //let sample = wavetable::SAW[wt_index as usize];
    let sample = interpolate_linear(&wavetable::SAW, wt_index);

    let dx = f / fs;
    let phase = phase + dx;
    let phase = if phase > 1.0 { phase - 1.0 } else { phase };

    return (phase, sample);
}


// - conversions --------------------------------------------------------------

#[inline(always)]
pub fn clip_convert_u8(x: f32) -> u8 {
    let x: f32 = if x > 1.0 { 1.0 } else if x < 1.0 { -1.0 } else { x };
    let x: f32 = x + 1.;
    //let x: f32 = x / 2.;
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
