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

const TAG: &str = "api::driver::sh1106::i2c";


// - i2c ----------------------------------------------------------------------

const ACK_CHECK_EN: bool  = true;  // I2C master will check ack from slave
const ACK_CHECK_DIS: bool = false; // I2C master will not check ack from slave
const ACK_VAL: bool       = false; // I2C ack value
const NACK_VAL: bool      = true;  // I2C nack value


// - initialization -----------------------------------------------------------

pub unsafe fn init(port: i2c_port_t, pins: Pins) -> Result<(), EspError> {
    log!(TAG, "configure pins for display peripheral i2c: {:?}", pins);
    let i2c_config = i2c_config_t {
        mode:  i2c_mode_t::I2C_MODE_MASTER,
        scl_io_num:  pins.scl,
        sda_io_num:  pins.sda,
        scl_pullup_en:  gpio_pullup_t::GPIO_PULLUP_ENABLE,
        sda_pullup_en:  gpio_pullup_t::GPIO_PULLUP_ENABLE,
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
    log!(TAG, "detecting sh1106 oled display...");

    // check if display is reachable over i2s
    let value = read(port, address, 0)?; //Register::CHIP_ID)?;
    log!(TAG, "SH1106 display driver chip ID: 0x{:x}", value);
    /*if (value & 0xFF00) != 0xA000 {
        log!(TAG, "unknown codec chip ID: 0x{:x}", value);
        return Err(idf::ESP_ERR_INVALID_RESPONSE.into());
    }
    log!(TAG, "SH1106 display driver chip ID: 0x{:x}", value);*/

    Ok(())
}


// - read ---------------------------------------------------------------------

unsafe fn read(port: i2c_port_t, address: u8, register: Register) -> Result<u16, EspError> {
    let register: u16 = register.into();

    let cmd: i2c_cmd_handle_t = i2c_cmd_link_create();

    // start
    i2c_master_start(cmd).as_result()?;

    // set write bit for address
    i2c_master_write_byte(cmd, (address << 1) | i2c_rw_t::I2C_MASTER_WRITE as u8, ACK_CHECK_EN).as_result()?;

    // write register address
    i2c_master_write_byte(cmd, (register >> 8) as u8 & 0xff, ACK_CHECK_EN).as_result()?; // msb
    i2c_master_write_byte(cmd, (register & 0xff) as u8, ACK_CHECK_EN).as_result()?;      // lsb
    //let mut register: [u8; 2] = u16::to_le_bytes(register.into());
    //i2c_master_write(cmd, register.as_mut_ptr(), 2, ACK_CHECK_EN).as_result()?;

    // restart
    i2c_master_start(cmd).as_result()?;

    // set read bit for address
    i2c_master_write_byte(cmd, (address << 1) | i2c_rw_t::I2C_MASTER_READ as u8, ACK_CHECK_EN).as_result()?;

    // read register value
    let mut msb: u8 = 0;
    let mut lsb: u8 = 0;
    i2c_master_read_byte(cmd, &mut msb as *mut u8, i2c_ack_type_t::I2C_MASTER_ACK);  // msb
    i2c_master_read_byte(cmd, &mut lsb as *mut u8, i2c_ack_type_t::I2C_MASTER_NACK); // lsb

    // stop
    i2c_master_stop(cmd).as_result()?;

    // send
    i2c_master_cmd_begin(port, cmd, 1000 / portTICK_RATE_MS).as_result()?;
    i2c_cmd_link_delete(cmd);

    // get register value
    let value: u16 = ((msb as u16) << 8) | (lsb as u16);

    Ok(value)
}


// - write --------------------------------------------------------------------

unsafe fn write(port: i2c_port_t, address: u8, register: Register, value: u16) -> Result<(), EspError> {
    let cmd: i2c_cmd_handle_t = i2c_cmd_link_create();

    // start
    i2c_master_start(cmd).as_result()?;

    // set write bit for address
    i2c_master_write_byte(cmd, (address << 1) | i2c_rw_t::I2C_MASTER_WRITE as u8, ACK_CHECK_EN).as_result()?;

    // write value to register
    let register: u16 = register.into();
    let value: u16 = value.into();
    let mut bytes: [u8; 4] = [
        ((register >> 8) & 0x00ff) as u8,
        (register & 0x00ff) as u8,
        ((value >> 8) & 0x00ff) as u8,
        (value & 0x00ff) as u8,
    ];
    //let register: [u8; 2] = u16::to_le_bytes(register.into());
    //let value: [u8; 2] = u16::to_le_bytes(value.into());
    //let bytes: [u8; 4] = [ register[0], register[1], value[0], value[1] ];
    i2c_master_write(cmd, bytes.as_mut_ptr(), 4, ACK_CHECK_EN).as_result()?;

    // stop
    i2c_master_stop(cmd).as_result()?;

    // send
    i2c_master_cmd_begin(port, cmd, 1000 / portTICK_RATE_MS).as_result()?;
    i2c_cmd_link_delete(cmd);

    Ok(())
}




// - codec register addresses -------------------------------------------------

type Register = u16;
