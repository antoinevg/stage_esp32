use esp_idf::bindings as idf;
use esp_idf::{AsResult, EspError};

use crate::logger;


// - global constants ---------------------------------------------------------

const TAG: &str = "wrap::ledc";

const LEDC_BASE_FREQ: u32 = 50_000; // KHz


// - exports ------------------------------------------------------------------

pub unsafe fn init(gpios: &[u8]) -> Result<(), EspError> {
    if gpios.len() >= idf::ledc_channel_t::LEDC_CHANNEL_MAX as usize {
        log!(TAG, "Too many gpios for ledc peripheral. Maximum allowed is: {:?}", idf::ledc_channel_t::LEDC_CHANNEL_MAX);
        return Err(EspError(idf::ESP_ERR_INVALID_ARG as i32));
    }

    log!(TAG, "initializing ledc pwm driver");

    let timer = idf::ledc_timer_config_t {
        duty_resolution: idf::ledc_timer_bit_t::LEDC_TIMER_8_BIT, // resolution of PWM duty cycle
        freq_hz: LEDC_BASE_FREQ,                                  // frequency of PWM signal
        speed_mode: idf::ledc_mode_t::LEDC_HIGH_SPEED_MODE,       // timer mode
        timer_num: idf::ledc_timer_t::LEDC_TIMER_0,               // timer index
        ..idf::ledc_timer_config_t::default()
    };
    idf::ledc_timer_config(&timer);

    // configure ledc pwm channels
    for (channel, gpio) in gpios.iter().enumerate() {
        let channel = core::mem::transmute::<usize, idf::ledc_channel_t>(channel);
        let config = idf::ledc_channel_config_t {
            channel: channel,
            duty: 0,
            gpio_num: *gpio as i32,
            speed_mode: idf::ledc_mode_t::LEDC_HIGH_SPEED_MODE,
            hpoint: 0,
            timer_sel: idf::ledc_timer_t::LEDC_TIMER_0,
            ..idf::ledc_channel_config_t::default()
        };
        log!(TAG, "configuring {:?} -> GPIO_{}", channel, gpio);
        idf::ledc_channel_config(&config);
    }

    idf::ledc_fade_func_install(0);

    // set default values
    for (channel, _)  in gpios.iter().enumerate() {
        let channel = core::mem::transmute::<usize, idf::ledc_channel_t>(channel);
        idf::ledc_set_duty(idf::ledc_mode_t::LEDC_HIGH_SPEED_MODE, channel, 255);
        idf::ledc_update_duty(idf::ledc_mode_t::LEDC_HIGH_SPEED_MODE, channel);
    }

    Ok(())
}


pub fn update(channel: u8, value: f32) {
    let channel = unsafe { core::mem::transmute::<u32, idf::ledc_channel_t>(channel as u32) };
    let duty = (value * 255.) as u32;
    unsafe {
        idf::ledc_set_duty(idf::ledc_mode_t::LEDC_HIGH_SPEED_MODE, channel, duty);
        idf::ledc_update_duty(idf::ledc_mode_t::LEDC_HIGH_SPEED_MODE, channel);
    }
}
