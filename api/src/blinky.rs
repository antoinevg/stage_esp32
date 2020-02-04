// - registers ----------------------------------------------------------------

// https://github.com/espressif/esp-idf/blob/master/components/soc/esp32/include/soc/gpio_reg.h
mod gpio_reg {
    pub const GPIO_ENABLE_W1TS_REG: u32 = 0x3FF44024;
    pub const GPIO_OUT_W1TS_REG: u32 = 0x3FF44008;
    pub const GPIO_OUT_W1TC_REG : u32 = 0x3FF4400C;
    pub const GPIO_FUNCX_OUT_BASE: u32 = 0x3FF44530;
}


// - implementation -----------------------------------------------------------

pub fn set_led(idx: u8, val: bool) {
    if val {
        unsafe {
            core::ptr::write_volatile(gpio_reg::GPIO_OUT_W1TS_REG as *mut u32, 0x1 << idx);
        }
    } else {
       unsafe {
            core::ptr::write_volatile(gpio_reg::GPIO_OUT_W1TC_REG as *mut u32, 0x1 << idx);
        }
    }
}


pub fn configure_pin_as_output(gpio: u8) {
    unsafe {
        core::ptr::write_volatile(gpio_reg::GPIO_ENABLE_W1TS_REG as *mut _, 0x1 << (gpio as u32));

        let gpio_funcx_out_sel_cfg: u32 = gpio_reg::GPIO_FUNCX_OUT_BASE + ((gpio as u32) * 4);
        core::ptr::write_volatile(gpio_funcx_out_sel_cfg as *mut _, 0x100);
    }
}


pub fn delay(clocks: u32) {
    let target = get_ccount() + clocks;
    loop {
        if get_ccount() > target {
            break;
        }
    }
}


fn get_ccount() -> u32 {
    let x: u32;
    unsafe {
        asm!("rsr.ccount a2" : "={a2}" (x))
    };
    x
}
