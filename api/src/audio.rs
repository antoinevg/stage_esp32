extern crate alloc;
use alloc::boxed::Box;

use cty::{c_int, c_void};

use esp_idf::bindings as idf;
use esp_idf::{AsResult, EspError, portMAX_DELAY, portMUX_INITIALIZER_UNLOCKED};

use crate::logger;
use crate::wavetable;

use crate::driver::Codec;
use crate::driver::adac;
use crate::driver::sgtl5000;


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
    pub codec: D,
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
        // initialize codec
        self.codec.init(&mut self.config)?;

        // set up audio callback
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

        // start audio thread
        let stack_depth = 8192;
        let priority = 5;
        let core_id = 1;
        let mut task_handle = unsafe { core::mem::zeroed::<c_void>() };
        let task_handle_ptr = &mut task_handle as *mut _ as *mut idf::TaskHandle_t;
        unsafe {
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

    fn audio_thread(&mut self) {
        const TAG: &str = "api::audio::thread";

        log!(TAG, "unused stack memory: {} bytes", unsafe {
            idf::uxTaskGetStackHighWaterMark(core::ptr::null_mut())
        });

        let Config { fs, num_channels, word_size, block_size } = self.config;
        let buffer_size = block_size * word_size;
        let num_frames  = block_size / num_channels;

        // allocate memory for callback buffer
        // TODO try `heap_caps_aligned_alloc` once we can build against esp-idf master again
        //      https://docs.espressif.com/projects/esp-idf/en/latest/api-reference/system/mem_alloc.html
        let buffer_ptr = unsafe {
            idf::calloc(block_size as u32,
                        core::mem::size_of::<f32>() as u32) as *mut f32
        };
        if buffer_ptr == core::ptr::null_mut() {
            panic!("api::audio::thread failed to allocate memory for callback buffer");
        }
        log!(TAG, "allocated memory for callback buffer: {} bytes", buffer_size);
        log!(TAG, "starting audio with fs: {} blocksize: {}", fs, block_size);

        //let mut state = State::new();
        let mut mux: idf::portMUX_TYPE = portMUX_INITIALIZER_UNLOCKED;
        loop {
            let mut buffer: &mut Buffer = unsafe {
                core::slice::from_raw_parts_mut(buffer_ptr, block_size)
            };

            // read buffer from driver
            match self.codec.read(&self.config, &mut buffer) {
                Ok(()) => (),
                Err(EspError(e)) => {
                    log!(TAG, "codec.read failed with: {:?}", e);
                }
            }

            // pass buffer to audio callback
            unsafe { idf::vTaskEnterCritical(&mut mux); }
            /*for f in 0..num_frames {
                let x = f * num_channels;
                state.channel_1 = test_signal_sin(fs, 1000., state.channel_1.0);
                state.channel_2 = test_signal_saw(fs, 1000., state.channel_2.0);
                buffer[x+0] = state.channel_2.1; // right
                buffer[x+1] = state.channel_1.1; // left
            }*/
            //test_callback_inline(fs, num_channels, buffer, &mut state);
            //test_callback(fs, num_channels, buffer, &mut state);
            (self.closure)(fs, num_channels, buffer);
            unsafe { idf::vTaskExitCritical(&mut mux); }

            // write buffer to driver
            match self.codec.write(&self.config, &buffer) {
                Ok(()) => (),
                Err(EspError(e)) => {
                    log!(TAG, "codec.write failed with: {:?}", e);
                }
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

        state.channel_1 = test_signal_sin(fs, 1000., state.channel_1.0);
        state.channel_2 = test_signal_saw(fs, 1000., state.channel_2.0);

        buffer[x+0] = state.channel_2.1; // right
        buffer[x+1] = state.channel_1.1; // left
    }
}

#[inline(always)]
fn test_callback_inline(fs: f32, num_channels: usize, buffer: &mut Buffer, state: &mut State) {
    let num_frames = buffer.len() / num_channels;
    for f in 0..num_frames {
        let x = f * num_channels;

        state.channel_1 = test_signal_sin(fs, 1000., state.channel_1.0);
        state.channel_2 = test_signal_saw(fs, 1000., state.channel_2.0);

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
