use num_enum::IntoPrimitive;

use esp_idf::{AsResult, EspError, portMAX_DELAY, portTICK_RATE_MS};
use esp_idf::bindings::{
    gpio_num_t,
    gpio_pullup_t,
    i2c_port_t,
    i2c_config_t,
    i2c_mode_t,
    i2c_cmd_handle_t,
    i2c_rw_t,
    i2c_ack_type_t,
};
use esp_idf::bindings::{
    i2c_param_config,
    i2c_driver_install,
    i2c_cmd_link_create,
    i2c_cmd_link_delete,
    i2c_master_cmd_begin,
    i2c_master_start,
    i2c_master_stop,
    i2c_master_read,
    i2c_master_read_byte,
    i2c_master_write,
    i2c_master_write_byte,
};
use esp_idf::bindings::{
    ESP_INTR_FLAG_LEVEL1,
};
use esp_idf::bindings as idf;

use crate::i2c::{Pins};
use crate::logger;


// - global constants ---------------------------------------------------------

const TAG: &str = "api::driver::wm8731::i2c";


// - i2c ----------------------------------------------------------------------

const ACK_CHECK_EN: bool  = true;  // I2C master will check ack from slave
const ACK_CHECK_DIS: bool = false; // I2C master will not check ack from slave
const ACK_VAL: bool       = false; // I2C ack value
const NACK_VAL: bool      = true;  // I2C nack value


// - initialization -----------------------------------------------------------

pub unsafe fn init(port: i2c_port_t, pins: Pins) -> Result<(), EspError> {
    log!(TAG, "configure pins for codec peripheral i2c: {:?}", pins);
    let i2c_config = i2c_config_t {
        mode:  i2c_mode_t::I2C_MODE_MASTER,
        scl_io_num:  pins.scl,
        sda_io_num:  pins.sda,
        scl_pullup_en:  gpio_pullup_t::GPIO_PULLUP_DISABLE,
        sda_pullup_en:  gpio_pullup_t::GPIO_PULLUP_DISABLE,
        __bindgen_anon_1: idf::i2c_config_t__bindgen_ty_1 {
            master: idf::i2c_config_t__bindgen_ty_1__bindgen_ty_1 {
                clk_speed: 100_000
            }
        }
    };
    i2c_param_config(port, &i2c_config).as_result()?;
    i2c_driver_install(port, i2c_config.mode,
                       0, // rx buffer length (slave only)
                       0, // tx buffer length (slave only)
                       0).as_result()?; //ESP_INTR_FLAG_LEVEL1 as i32).as_result()?;

    Ok(())
}


pub unsafe fn configure(port: i2c_port_t, address: u8) -> Result<(), EspError> {
    idf::vTaskDelay(1);

    for (register, value) in REGISTER_CONFIG {
        log!(TAG, "Configure register {:?}: 0x{:x}", register, value);

        let register: u8 = (*register).into();
        let byte1: u8 = (register << 1) | (value >> 8);
        let byte2: u8 = value & 0xFF;

        let cmd: i2c_cmd_handle_t = i2c_cmd_link_create();
        i2c_master_start(cmd).as_result()?;
        i2c_master_write_byte(cmd, (address << 1) | i2c_rw_t::I2C_MASTER_WRITE as u8, ACK_CHECK_EN).as_result()?;
        i2c_master_write_byte(cmd, byte1, ACK_CHECK_EN).as_result()?;
        i2c_master_write_byte(cmd, byte2, ACK_CHECK_EN).as_result()?;
        i2c_master_stop(cmd).as_result()?;
        i2c_master_cmd_begin(port, cmd, 1000 / portTICK_RATE_MS).as_result()?;
        i2c_cmd_link_delete(cmd);

        idf::vTaskDelay(1);
    }

    Ok(())
}



// - codec register addresses -------------------------------------------------

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, IntoPrimitive)]
#[repr(u8)]
enum Register {
    LINVOL = 0x00,
    RINVOL = 0x01,
    LOUT1V = 0x02,
    ROUT1V = 0x03,
    APANA  = 0x04,
    APDIGI = 0x05, // 0000_0101
    PWR    = 0x06,
    IFACE  = 0x07, // 0000_0111
    SRATE  = 0x08, // 0000_1000
    ACTIVE = 0x09, // 0000_1001
    RESET  = 0x0F,
}


const REGISTER_CONFIG: &[(Register, u8)] = &[
    (Register::PWR,    0x80),
    (Register::RESET,  0x00),
    (Register::ACTIVE, 0x00),
    (Register::APANA,  0x12),
    //(Register::APANA,  0b0001_0010), // MICBOOST=0 MUTEMIC=1 INSEL=0 BYPASS=0 DACSEL=1 SIDETONE=0
    (Register::APDIGI, 0x00),
    (Register::PWR,    0x00),
    //(Register::IFACE,  0x02),
    (Register::IFACE,  0b0100_0010), // 0x40 FORMAT=b10 IRL=b00 LRP=0 LRSWAP=0 MS=1 BCKLINV=0
    //(Register::IFACE,  0b0000_0010), // 0x40 FORMAT=b10 IRL=b00 LRP=0 LRSWAP=0 MS=0 BCKLINV=0
    //(Register::SRATE,  0b0000_0000), // MODE=0 BOSR=0 FS=48Khz CLKIDIV2=0 CLKODIV2=0
    (Register::SRATE,  0b0000_0001), // MODE=1 BOSR=0 FS=48Khz CLKIDIV2=0 CLKODIV2=0
    (Register::LINVOL, 0x17),
    (Register::RINVOL, 0x17),
    (Register::LOUT1V, 0x7F),
    (Register::ROUT1V, 0x7F),
    (Register::ACTIVE, 0x01),
];
