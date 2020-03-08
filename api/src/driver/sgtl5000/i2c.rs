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

const TAG: &str = "api::driver::sgtl5000::i2c";


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
    log!(TAG, "detecting sgtl5000 audio codec...");

    // check if codec is reachable over i2s
    let value = read(port, address, Register::CHIP_ID)?;
    if (value & 0xFF00) != 0xA000 {
        log!(TAG, "unknown codec chip ID: 0x{:x}", value);
        return Err(idf::ESP_ERR_INVALID_RESPONSE.into());
    }
    log!(TAG, "SGTL5000 codec chip ID: 0x{:x}", value);

    // set register defaults
    for (register, value) in REGISTER_DEFAULTS {
        write(port, address, *register, *value)?;
    }
    log!(TAG, "set register defaults");

    // chip powerup and supply configurations
    write(port, address, Register::CHIP_ANA_POWER, 0x4260)?;
    log!(TAG, "turned off startup power supplies to save power");
    write(port, address, Register::CHIP_LINREG_CTRL, 0x006c)?;
    log!(TAG, "configured the charge pump to use the VDDIO rail");
    write(port, address, Register::CHIP_REF_CTRL, 0x01f2)?;
    log!(TAG, "VAG=1.575, normal ramp, +12.5%% bias current");
    write(port, address, Register::CHIP_LINE_OUT_CTRL, 0x0322)?;
    log!(TAG, "set lineout power to 1.65V @ 0.36mA");
    write(port, address, Register::CHIP_SHORT_CTRL, 0x4446)?;
    log!(TAG, "allow up to 125mA for short control, err_code");
    write(port, address, Register::CHIP_ANA_POWER, 0x40ff)?;
    log!(TAG, "powered up analog power supplies, LINE OUT, HP");
    write(port, address, Register::CHIP_DIG_POWER, 0b1100011)?;
    log!(TAG, "powered up I2S_IN, I2S_OUT, ADC and DAC");
    write(port, address, Register::CHIP_LINE_OUT_VOL, 0x0f0f)?;
    log!(TAG, "set line out level to 3.2V peak-to-peak");

    // system MCLK and sample clock
    /*modify(port, address, Register::CHIP_CLK_CTRL, 5, 4, 0b00)?;        // bits 5:4
      log!(TAG, "configured RATE_MODE to SYSFS * 1");
      modify(port, address, Register::CHIP_CLK_CTRL, 3, 2, 0b10)?;        // bits 3:2
      log!(TAG, "configured SYS_FS clock to 48 kHz");
      modify(port, address, Register::CHIP_CLK_CTRL, 1, 0, 0b00)?;        // bits 1:0
      log!(TAG, "configured MCLK_FREQ to 256*Fs");*/
    modify(port, address, Register::CHIP_CLK_CTRL, 5, 0, 0b001000)?;        // bits 5:0
    log!(TAG, "configured CHIP_CLK_CTRL for SYSFS*1, 48kHz, 256*Fs");

    // i2s configuration
    modify(port, address, Register::CHIP_I2S_CTRL, 8, 0, 0b100110000)?; // bits 8:0
    log!(TAG, "configured i2s for SCLK=32*Fs 16bits, I2S slave, no PLL ");

    // input/output routing
    modify(port, address, Register::CHIP_SSS_CTRL, 5, 4, 0b01)?; // bits 5:4
    log!(TAG, "attach I2S_IN to DAC");
    modify(port, address, Register::CHIP_SSS_CTRL, 1, 0, 0b00)?; // bits 1:0
    log!(TAG, "attach I2S_OUT to ADC");

    // volume control
    modify(port, address, Register::CHIP_ADCDAC_CTRL, 0, 0, 0b1)?;  // bits 0:0
    log!(TAG, "disable high-pass filter");
    modify(port, address, Register::CHIP_ADCDAC_CTRL, 9, 8, 0b11)?; // bits 9:8
    log!(TAG, "enable exponential volume ramp");
    modify(port, address, Register::CHIP_ADCDAC_CTRL, 3, 2, 0b00)?; // bits 3:2
    log!(TAG, "unmute DAC");
    write(port, address, Register::CHIP_DAC_VOL, 0x3c3c)?;
    log!(TAG, "set DAC volume to -0.5dB");
    write(port, address, Register::CHIP_ANA_ADC_CTRL, 0x00)?;
    log!(TAG, "set ADC input level to 0dB");
    write(port, address, Register::CHIP_ANA_HP_CTRL, 0x3a3a)?;
    log!(TAG, "set HP out volume to -17dB");
    modify(port, address, Register::CHIP_ANA_CTRL, 8, 0, 0b000000100)?;
    log!(TAG, "unmute line and headphone outputs");

    //dump_registers(port, address)?;

    Ok(())
}


fn dump_registers(port: i2c_port_t, address: u8) -> Result<(), EspError> {
    const REGISTERS: &[Register] = &[
        Register::CHIP_ID,
        Register::CHIP_ANA_POWER,
        Register::CHIP_LINREG_CTRL,
        Register::CHIP_REF_CTRL,
        Register::CHIP_LINE_OUT_CTRL,
        Register::CHIP_SHORT_CTRL,
        Register::CHIP_DIG_POWER,
        Register::CHIP_LINE_OUT_VOL,
        Register::CHIP_CLK_CTRL,
        Register::CHIP_I2S_CTRL,
        Register::CHIP_SSS_CTRL,
        Register::CHIP_ADCDAC_CTRL,
        Register::CHIP_DAC_VOL,
        Register::CHIP_ANA_ADC_CTRL,
        Register::CHIP_ANA_HP_CTRL,
        Register::CHIP_ANA_CTRL,
    ];

    for register in REGISTERS {
        let value = unsafe { read(port, address, *register)? };
        let register: u16 = (*register).into();
        log!(TAG, "0x{:x}\t0x{:x}", register, value);
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


// - modify -------------------------------------------------------------------

unsafe fn modify(port: i2c_port_t, address: u8,
                 register: Register,
                 bit_hi: u16, bit_lo: u16, value: u16) -> Result<(), EspError> {

    let current = read(port, address, register)?;

    let width = (bit_hi - bit_lo) + 1;
    let position = bit_lo;
    let mask = ((2 << (width - 1)) - 1) << position;

    let write_value = (current & (!mask)) | (value << position);

    write(port, address, register, write_value)
}


// - codec register addresses -------------------------------------------------

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, IntoPrimitive)]
#[repr(u16)]
enum Register {
    CHIP_ID                   = 0x0000,
    CHIP_DIG_POWER            = 0x0002,
    CHIP_CLK_CTRL             = 0x0004,
    CHIP_I2S_CTRL             = 0x0006,
    CHIP_SSS_CTRL             = 0x000A,
    CHIP_ADCDAC_CTRL          = 0x000E,
    CHIP_DAC_VOL              = 0x0010,
    CHIP_PAD_STRENGTH         = 0x0014,
    CHIP_ANA_ADC_CTRL         = 0x0020,
    CHIP_ANA_HP_CTRL          = 0x0022,
    CHIP_ANA_CTRL             = 0x0024,
    CHIP_LINREG_CTRL          = 0x0026,
    CHIP_REF_CTRL             = 0x0028,
    CHIP_MIC_CTRL             = 0x002A,
    CHIP_LINE_OUT_CTRL        = 0x002C,
    CHIP_LINE_OUT_VOL         = 0x002E,
    CHIP_ANA_POWER            = 0x0030,
    CHIP_PLL_CTRL             = 0x0032,
    CHIP_CLK_TOP_CTRL         = 0x0034,
    CHIP_ANA_STATUS           = 0x0036,
    CHIP_ANA_TEST1            = 0x0038,
    CHIP_ANA_TEST2            = 0x003A,
    CHIP_SHORT_CTRL           = 0x003C,
    DAP_CTRL                  = 0x0100,
    DAP_PEQ                   = 0x0102,
    DAP_BASS_ENHANCE          = 0x0104,
    DAP_BASS_ENHANCE_CTRL     = 0x0106,
    DAP_AUDIO_EQ              = 0x0108,
    DAP_SGTL_SURROUND         = 0x010A,
    DAP_FILTER_COEF_ACCESS    = 0x010C,
    DAP_COEF_WR_B0_MSB        = 0x010E,
    DAP_COEF_WR_B0_LSB        = 0x0110,
    DAP_AUDIO_EQ_BASS_BAND0   = 0x0116,
    DAP_AUDIO_EQ_BAND1        = 0x0118,
    DAP_AUDIO_EQ_BAND2        = 0x011A,
    DAP_AUDIO_EQ_BAND3        = 0x011C,
    DAP_AUDIO_EQ_TREBLE_BAND4 = 0x011E,
    DAP_MAIN_CHAN             = 0x0120,
    DAP_MIX_CHAN              = 0x0122,
    DAP_AVC_CTRL              = 0x0124,
    DAP_AVC_THRESHOLD         = 0x0126,
    DAP_AVC_ATTACK            = 0x0128,
    DAP_AVC_DECAY             = 0x012A,
    DAP_COEF_WR_B1_MSB        = 0x012C,
    DAP_COEF_WR_B1_LSB        = 0x012E,
    DAP_COEF_WR_B2_MSB        = 0x0130,
    DAP_COEF_WR_B2_LSB        = 0x0132,
    DAP_COEF_WR_A1_MSB        = 0x0134,
    DAP_COEF_WR_A1_LSB        = 0x0136,
    DAP_COEF_WR_A2_MSB        = 0x0138,
    DAP_COEF_WR_A2_LSB        = 0x013A
}

const REGISTER_DEFAULTS: &[(Register, u16)] = &[
    (Register::CHIP_DIG_POWER,            0x0000),
    (Register::CHIP_I2S_CTRL,             0x0010),
    (Register::CHIP_SSS_CTRL,             0x0010),
    (Register::CHIP_ADCDAC_CTRL,          0x020c),
    (Register::CHIP_DAC_VOL,              0x3c3c),
    (Register::CHIP_PAD_STRENGTH,         0x015f),
    (Register::CHIP_ANA_ADC_CTRL,         0x0000),
    (Register::CHIP_ANA_HP_CTRL,          0x1818),
    (Register::CHIP_ANA_CTRL,             0x0111),
    (Register::CHIP_REF_CTRL,             0x0000),
    (Register::CHIP_MIC_CTRL,             0x0000),
    (Register::CHIP_LINE_OUT_CTRL,        0x0000),
    (Register::CHIP_LINE_OUT_VOL,         0x0404),
    (Register::CHIP_PLL_CTRL,             0x5000),
    (Register::CHIP_CLK_TOP_CTRL,         0x0000),
    (Register::CHIP_ANA_STATUS,           0x0000),
    (Register::CHIP_SHORT_CTRL,           0x0000),
    (Register::CHIP_ANA_TEST2,            0x0000),
    (Register::DAP_CTRL,                  0x0000),
    (Register::DAP_PEQ,                   0x0000),
    (Register::DAP_BASS_ENHANCE,          0x0040),
    (Register::DAP_BASS_ENHANCE_CTRL,     0x051f),
    (Register::DAP_AUDIO_EQ,              0x0000),
    (Register::DAP_SGTL_SURROUND,         0x0040),
    (Register::DAP_AUDIO_EQ_BASS_BAND0,   0x002f),
    (Register::DAP_AUDIO_EQ_BAND1,        0x002f),
    (Register::DAP_AUDIO_EQ_BAND2,        0x002f),
    (Register::DAP_AUDIO_EQ_BAND3,        0x002f),
    (Register::DAP_AUDIO_EQ_TREBLE_BAND4, 0x002f),
    (Register::DAP_MAIN_CHAN,             0x8000),
    (Register::DAP_MIX_CHAN,              0x0000),
    (Register::DAP_AVC_CTRL,              0x0510),
    (Register::DAP_AVC_THRESHOLD,         0x1473),
    (Register::DAP_AVC_ATTACK,            0x0028),
    (Register::DAP_AVC_DECAY,             0x0050),
];
