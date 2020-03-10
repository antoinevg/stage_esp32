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

use crate::blinky;
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

    // TODO i2c init should be happening in api/i2c

    /*
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
     */
    Ok(())
}


pub unsafe fn configure(reset: gpio_num_t, port: i2c_port_t, address: u8) -> Result<(), EspError> {
    // also see: https://github.com/Devilbinder/SH1106/blob/master/SH1106.cpp

    // reset display
    log!(TAG, "resetting display peripheral");
    let delay = (0.001 * 168_000_000.) as u32;
    blinky::configure_pin_as_output(reset)?;
    blinky::set_led(reset, true)?;
    blinky::delay(delay * 10);
    blinky::set_led(reset, false)?;
    blinky::delay(delay * 200);
    blinky::set_led(reset, true)?;
    blinky::delay(delay * 10);

    // TODO check if display is reachable over i2s
    /*let value = read(port, address, 0x00); //Register::CHIP_ID)?;
    if (value & 0xFF00) != 0x3c00 {
        log!(TAG, "unknown display chip ID: 0x{:x}", value);
        return Err(idf::ESP_ERR_INVALID_RESPONSE.into());
    }
    log!(TAG, "SH1106 display driver chip ID: 0x{:x}", value);*/

    log!(TAG, "configuring sh1106 oled display at address: 0x{:x}", address);
    write(port, address, 0x00, 0xAE)?; // turn off oled panel
    write(port, address, 0x00, 0x02)?; // set low column address
    write(port, address, 0x00, 0x10)?; // set high column address
    write(port, address, 0x00, 0x40)?; // set start line address  Set Mapping RAM Display Start Line (0x00~0x3F)
    write(port, address, 0x00, 0x81)?; // set contrast control register
    write(port, address, 0x00, 0xA0)?; // Set SEG/Column Mapping
    write(port, address, 0x00, 0xC0)?; // Set COM/Row Scan Direction
    write(port, address, 0x00, 0xA6)?; // set normal display
    write(port, address, 0x00, 0xA8)?; // set multiplex ratio(1 to 64)
    write(port, address, 0x00, 0x3F)?; // 1/64 duty
    write(port, address, 0x00, 0xD3)?; // set display offset    Shift Mapping RAM Counter (0x00~0x3F)
    write(port, address, 0x00, 0x00)?; // not offset
    write(port, address, 0x00, 0xd5)?; // set display clock divide ratio/oscillator frequency
    write(port, address, 0x00, 0x80)?; // set divide ratio, Set Clock as 100 Frames/Sec
    write(port, address, 0x00, 0xD9)?; // set pre-charge period
    write(port, address, 0x00, 0xF1)?; // Set Pre-Charge as 15 Clocks & Discharge as 1 Clock
    write(port, address, 0x00, 0xDA)?; // set com pins hardware configuration
    write(port, address, 0x00, 0x12)?;
    write(port, address, 0x00, 0xDB)?; // set vcomh
    write(port, address, 0x00, 0x40)?; // Set VCOM Deselect Level
    write(port, address, 0x00, 0x20)?; // Set Page Addressing Mode (0x00/0x01/0x02)
    write(port, address, 0x00, 0x02)?; //
    write(port, address, 0x00, 0xA4)?; // Disable Entire Display On (0xa4/0xa5)
    write(port, address, 0x00, 0xA6)?; // Disable Inverse Display On (0xa6/a7)

    blinky::delay(delay);
    write(port, address, 0x00, 0xAF)?; // turn on oled panel

    // allocate a page_buffer
    #[allow(non_upper_case_globals)] const width: usize = 128;
    #[allow(non_upper_case_globals)] const height: usize = 64;
    let mut page_buffer: [u8; width] = [0x00; width];

    // blank display
    log!(TAG, "clearing display");
    for page in 0usize..8 {
        let page_address = (0xb0 + page) as u8;
        write(port, address, 0x00, page_address)?;       // set page address
        write(port, address, 0x00, 0x02)?;               // set low column address
        write(port, address, 0x00, 0x10)?;               // set high column address
        write_bytes(port, address, 0x40, &page_buffer)?; // write data for page
    }

    blinky::delay(delay);

    // generate data for a test pattern
    for x in 0..width {
        let byte = if x % 8 == 0 { 255 } else { 1 };
        page_buffer[x] = byte;
    }

    // blit test pattern to the display
    log!(TAG, "display test pattern");
    for page in 0usize..8 {
        let page_address = (0xb0 + page) as u8;
        write(port, address, 0x00, page_address)?;       // set page address
        write(port, address, 0x00, 0x02)?;               // set low column address
        write(port, address, 0x00, 0x10)?;               // set high column address
        write_bytes(port, address, 0x40, &page_buffer)?; // write data for page
    }



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

unsafe fn write(port: i2c_port_t, address: u8, register: Register, byte: u8) -> Result<(), EspError> {
    let cmd: i2c_cmd_handle_t = i2c_cmd_link_create();

    // start
    i2c_master_start(cmd).as_result()?;

    // set write bit for address
    i2c_master_write_byte(cmd, (address << 1) | i2c_rw_t::I2C_MASTER_WRITE as u8, ACK_CHECK_EN).as_result()?;

    // write byte to register
    let register: u8 = register as u8; //.into();
    i2c_master_write_byte(cmd, register, ACK_CHECK_EN).as_result()?;
    i2c_master_write_byte(cmd, byte, ACK_CHECK_EN).as_result()?;

    // stop
    i2c_master_stop(cmd).as_result()?;

    // send
    i2c_master_cmd_begin(port, cmd, 1000 / portTICK_RATE_MS).as_result()?;
    i2c_cmd_link_delete(cmd);

    Ok(())
}


unsafe fn write_bytes(port: i2c_port_t, address: u8, register: Register, bytes: &[u8]) -> Result<(), EspError> {
    let cmd: i2c_cmd_handle_t = i2c_cmd_link_create();

    // start
    i2c_master_start(cmd).as_result()?;

    // set write bit for address
    i2c_master_write_byte(cmd, (address << 1) | i2c_rw_t::I2C_MASTER_WRITE as u8, ACK_CHECK_EN).as_result()?;

    // select register
    let register: u8 = register as u8; //.into();
    i2c_master_write_byte(cmd, register, ACK_CHECK_DIS).as_result()?;

    // write bytes to register
    let bytes_ptr = core::mem::transmute::<*const u8, *mut u8>(bytes.as_ptr());
    i2c_master_write(cmd, bytes_ptr, bytes.len(), ACK_CHECK_DIS).as_result()?;

    // stop
    i2c_master_stop(cmd).as_result()?;

    // send
    i2c_master_cmd_begin(port, cmd, 1000 / portTICK_RATE_MS).as_result()?;
    i2c_cmd_link_delete(cmd);

    Ok(())
}


unsafe fn write_u16(port: i2c_port_t, address: u8, register: Register, value: u16) -> Result<(), EspError> {
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
