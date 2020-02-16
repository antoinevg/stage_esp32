use esp_idf::bindings::{
    ESP_ERR_INVALID_ARG
};
use esp_idf::bindings::{
    gpio_num_t,
    ledc_channel_config_t,
    ledc_channel_t,
    ledc_mode_t,
    ledc_timer_bit_t,
    ledc_timer_config_t,
    ledc_timer_t
};
use esp_idf::bindings::{
    ledc_channel_config,
    ledc_fade_func_install,
    ledc_set_duty,
    ledc_timer_config,
    ledc_update_duty,
};
use esp_idf::{AsResult, EspError};

use crate::logger;


// - global constants ---------------------------------------------------------

const TAG: &str = "api::ledc";

// the lower the pwm signal frequency, the higher the duty resolution:
//   40.00 MHz -> 1 bit
//   20.00 MHz -> 2 bits
//   10.00 MHz -> 3 bits
//    5.00 MHz -> 4 bits
//    2.50 MHz -> 5 bits
//    1.25 MHz -> 6 bits
//  625.00 kHz -> 7 bits
//  312.50 kHz -> 8 bits
//  156.25 kHz -> 9 bits
//   78.13 kHz -> 10 bits
//   39.06 kHz -> 11 bits
//   19.53 kHz -> 12 bits
//    9.76 kHz -> 13 bits
//    4.88 kHz -> 14 bits
//    2.44 kHz -> 15 bits
//    1.22 kHz -> 16 bits
//    ...
const FREQ_HZ: u32 = 19_500;
const DUTY_RESOLUTION: ledc_timer_bit_t = ledc_timer_bit_t::LEDC_TIMER_12_BIT;
pub const DUTY_SCALE: f32 = ((2 << ((DUTY_RESOLUTION as u32) - 1)) - 1) as f32;


// - exports ------------------------------------------------------------------

pub unsafe fn init(pins: &[gpio_num_t]) -> Result<(), EspError> {
    if pins.len() >= ledc_channel_t::LEDC_CHANNEL_MAX as usize {
        log!(TAG, "Too many pins for ledc peripheral. Maximum allowed is: {:?}",
             ledc_channel_t::LEDC_CHANNEL_MAX);
        return Err(ESP_ERR_INVALID_ARG.into());
    }

    log!(TAG, "initializing ledc pwm driver");

    let timer = ledc_timer_config_t {
        timer_num: ledc_timer_t::LEDC_TIMER_0,          // timer index
        speed_mode: ledc_mode_t::LEDC_HIGH_SPEED_MODE,  // timer mode
        freq_hz: FREQ_HZ,                                    // frequency of PWM signal
        duty_resolution: DUTY_RESOLUTION,                    // resolution of PWM duty cycle
        ..ledc_timer_config_t::default()
    };
    ledc_timer_config(&timer);

    // configure ledc pwm channels
    log!(TAG, "configure pins for ledc pwm driver: {:?}", pins);
    for (channel, pin) in pins.iter().enumerate() {
        let channel = core::mem::transmute::<usize, ledc_channel_t>(channel);
        let config = ledc_channel_config_t {
            channel: channel,
            duty: 0,
            gpio_num: *pin as i32,
            speed_mode: ledc_mode_t::LEDC_HIGH_SPEED_MODE,
            hpoint: 0,
            timer_sel: ledc_timer_t::LEDC_TIMER_0,
            ..ledc_channel_config_t::default()
        };
        log!(TAG, "configuring {:?} -> GPIO_{:?}", channel, pin);
        ledc_channel_config(&config);
    }

    ledc_fade_func_install(0);

    // set default duty-cycle to 25%
    for (channel, _)  in pins.iter().enumerate() {
        update(channel as u8, 0.25);
    }

    Ok(())
}


pub fn update(channel: u8, value: f32) {
    let channel = unsafe { core::mem::transmute::<u32, ledc_channel_t>(channel as u32) };
    let duty = (value * DUTY_SCALE) as u32;

    unsafe {
        ledc_set_duty(ledc_mode_t::LEDC_HIGH_SPEED_MODE, channel, duty);
        ledc_update_duty(ledc_mode_t::LEDC_HIGH_SPEED_MODE, channel);
    }
}
