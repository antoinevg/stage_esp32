extern crate alloc;
use alloc::boxed::Box;

use cty::{c_int, c_void};

use esp_idf::bindings as idf;
use esp_idf::{AsResult, EspError, portMAX_DELAY, portMUX_INITIALIZER_UNLOCKED};

use crate::driver;
use crate::logger;
use crate::wavetable;


// - global constants ---------------------------------------------------------

const TAG: &str = "api::audio";

const CODEC_NOTIFY_BIT_THREAD_READY: u32 = 0b01;
const CODEC_NOTIFY_BIT_CODEC_READY:  u32 = 0b10;


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
    pub block_length: usize,
}


pub struct Interface<'a, D> {
    pub config: Config,
    pub driver: D,
    pub closure: Box<dyn FnMut(f32, usize, &mut Buffer) + 'a>,

    task_thread: idf::TaskHandle_t,
    task_root:   idf::TaskHandle_t,
}


impl<'a, D> Interface<'a, D>
where D: driver::Codec {
    pub fn new<F: FnMut(f32, usize, &mut Buffer) + 'a>(fs: f32, block_length: usize, closure: F) -> Interface<'a, D> {
        Interface {
            config: Config {
                fs: fs,
                num_channels: 2,
                word_size: 2,
                block_length: block_length,
            },
            driver: D::new(),
            closure: Box::new(closure),
            task_thread: &mut unsafe { core::mem::zeroed::<c_void>() },
            task_root:   &mut unsafe { core::mem::zeroed::<c_void>() },
        }
    }

    pub fn start_c(&self) -> Result<(), EspError> {
        let opaque_interface_ptr = unsafe {
            core::mem::transmute::<*const Interface<D>,
                                   *const OpaqueInterface>(self)
        };
        self.driver.start_c(&self.config, opaque_interface_ptr)
    }

    pub fn start(&mut self) -> Result<(), EspError> {
        // get task handle
        self.task_root = unsafe { idf::xTaskGetCurrentTaskHandle() };

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
        //let mut task_handle = unsafe { core::mem::zeroed::<c_void>() };
        //let task_handle_ptr = &mut task_handle as *mut _ as *mut idf::TaskHandle_t;
        let task_handle_ptr = &mut self.task_thread as *mut _ as *mut idf::TaskHandle_t;
        unsafe {
            idf::xTaskCreatePinnedToCore(Some(_Stage_api_audio_task_closure_wrapper),
                                         "audio::thread".as_bytes().as_ptr() as *const i8,
                                         stack_depth,
                                         closure_ptr,
                                         priority,
                                         task_handle_ptr,
                                         core_id);
        }

        log!(TAG, "wait for audio thread startup to complete");
        let mut bits: u32 = 0;
        unsafe { idf::xTaskNotifyWait(0, 0, &mut bits, portMAX_DELAY); }

        // initialize driver
        self.driver.init(&mut self.config)?;

        // let audio thread know the driver is ready
        log!(TAG, "driver initialization is complete");
        unsafe {
            idf::xTaskNotify(self.task_thread, CODEC_NOTIFY_BIT_CODEC_READY,
                             idf::eNotifyAction::eSetValueWithOverwrite);
        }

        Ok(())
    }

    fn audio_thread(&mut self) {
        const TAG: &str = "api::audio::thread";

        log!(TAG, "unused stack memory: {} bytes", unsafe {
            idf::uxTaskGetStackHighWaterMark(core::ptr::null_mut())
        });

        let Config { fs, num_channels, word_size, block_length } = self.config;
        let buffer_size = block_length * word_size;
        let num_frames  = block_length / num_channels;

        // allocate memory for callback buffer
        // TODO try `heap_caps_aligned_alloc` once we can build against esp-idf master again
        //      https://docs.espressif.com/projects/esp-idf/en/latest/api-reference/system/mem_alloc.html
        let buffer_ptr = unsafe {
            idf::calloc(block_length as u32,
                        core::mem::size_of::<f32>() as u32) as *mut f32
        };
        if buffer_ptr == core::ptr::null_mut() {
            panic!("api::audio::thread failed to allocate memory for callback buffer");
        }
        log!(TAG, "allocated memory for callback buffer: {} bytes", buffer_size);

        // tell main task that the thread has started
        log!(TAG, "starting audio with fs: {} blocksize: {}", fs, block_length);
        unsafe {
            idf::xTaskNotify(self.task_root, CODEC_NOTIFY_BIT_THREAD_READY,
                             idf::eNotifyAction::eSetValueWithOverwrite);
        }

        // block until codec is ready
        log!(TAG, "wait for codec initialization to complete");
        let mut bits: u32 = 0;
        unsafe {
            idf::xTaskNotifyWait(0, 0, &mut bits, portMAX_DELAY);
        }

        //let mut state = State::new();
        let mut mux: idf::portMUX_TYPE = portMUX_INITIALIZER_UNLOCKED;
        loop {
            let mut buffer: &mut Buffer = unsafe {
                core::slice::from_raw_parts_mut(buffer_ptr, block_length)
            };

            // read buffer from driver
            match self.driver.read(&self.config, &mut buffer) {
                Ok(()) => (),
                Err(EspError(e)) => {
                    log!(TAG, "driver.read failed with: {:?}", e);
                }
            }

            // pass buffer to audio callback
            unsafe { idf::vTaskEnterCritical(&mut mux); }
            /*for f in 0..num_frames {
                let x = f * num_channels;
                state.channel_1 = testsignal_sin(fs, 1000., state.channel_1.0);
                state.channel_2 = testsignal_saw(fs, 1000., state.channel_2.0);
                buffer[x+0] = state.channel_2.1; // right
                buffer[x+1] = state.channel_1.1; // left
            }*/
            //test_callback_inline(fs, num_channels, buffer, &mut state);
            //test_callback(fs, num_channels, buffer, &mut state);
            (self.closure)(fs, num_channels, buffer);
            unsafe { idf::vTaskExitCritical(&mut mux); }

            // write buffer to driver
            match self.driver.write(&self.config, &buffer) {
                Ok(()) => (),
                Err(EspError(e)) => {
                    log!(TAG, "driver.write failed with: {:?}", e);
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

        state.channel_1 = testsignal_sin(fs, 1000., state.channel_1.0);
        state.channel_2 = testsignal_saw(fs, 1000., state.channel_2.0);

        buffer[x+0] = state.channel_2.1; // right
        buffer[x+1] = state.channel_1.1; // left
    }
}

#[inline(always)]
fn test_callback_inline(fs: f32, num_channels: usize, buffer: &mut Buffer, state: &mut State) {
    let num_frames = buffer.len() / num_channels;
    for f in 0..num_frames {
        let x = f * num_channels;

        state.channel_1 = testsignal_sin(fs, 1000., state.channel_1.0);
        state.channel_2 = testsignal_saw(fs, 1000., state.channel_2.0);

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
pub fn testsignal(fs: f32, f: f32, phase: f32) -> (f32, f32) {
    let sample = (phase * 2.) - 1.;

    let dx = f / fs;
    let phase = phase + dx;
    let phase = if phase > 1. { phase - 1. } else { phase };

    return (phase, sample);
}


#[inline(always)]
pub fn testsignal_cosf(fs: f32, f: f32, phase: f32) -> (f32, f32) {
    let dx = f / fs;
    let w = 2. * PI * dx;

    let sample = unsafe { idf::cosf(phase * w) };

    let phase = phase + 1.;

    return (phase, sample);
}


#[inline(always)]
pub fn testsignal_sin(fs: f32, f: f32, phase: f32) -> (f32, f32) {
    let wt_index = phase * (wavetable::LENGTH - 1) as f32;
    //let sample = wavetable::SIN[wt_index as usize];
    let sample = interpolate_linear(&wavetable::SIN, wt_index);

    let dx = f / fs;
    let phase = phase + dx;
    let phase = if phase > 1.0 { phase - 1.0 } else { phase };

    return (phase, sample);
}


#[inline(always)]
pub fn testsignal_saw(fs: f32, f: f32, phase: f32) -> (f32, f32) {
    let wt_index = phase * (wavetable::LENGTH - 1) as f32;
    //let sample = wavetable::SAW[wt_index as usize];
    let sample = interpolate_linear(&wavetable::SAW, wt_index);

    let dx = f / fs;
    let phase = phase + dx;
    let phase = if phase > 1.0 { phase - 1.0 } else { phase };

    return (phase, sample);
}
