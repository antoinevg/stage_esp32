use num_enum::IntoPrimitive;

use esp_idf::{AsResult, EspError, portMAX_DELAY, portTICK_RATE_MS};
use esp_idf::bindings::{
    i2c_port_t,
    i2c_cmd_handle_t,
    i2c_rw_t,
    i2c_ack_type_t,
};
use esp_idf::bindings::{
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


// - codec register addresses -------------------------------------------------

#[allow(non_camel_case_types)]
#[derive(IntoPrimitive)]
#[repr(u16)]
enum Address {
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


// - i2c ----------------------------------------------------------------------

const ACK_CHECK_EN: bool  = true;  // I2C master will check ack from slave
const ACK_CHECK_DIS: bool = false; // I2C master will not check ack from slave
const ACK_VAL: bool       = false; // I2C ack value
const NACK_VAL: bool      = true;  // I2C nack value


fn foo() {
    let plonk: u16 = Address::CHIP_ID.into();
}

unsafe fn read(port: i2c_port_t, address: u8, register: Address) -> Result<u16, EspError> {
    let cmd: i2c_cmd_handle_t = i2c_cmd_link_create();

    // start
    i2c_master_start(cmd).as_result()?;

    // set write bit for address
    i2c_master_write_byte(cmd, (address << 1) | i2c_rw_t::I2C_MASTER_WRITE as u8, ACK_CHECK_EN).as_result()?;

    // write register address
    i2c_master_write_byte(cmd, (address >> 8) & 0xFF, ACK_CHECK_EN).as_result()?; // msb
    i2c_master_write_byte(cmd, address & 0xFF, ACK_CHECK_EN).as_result()?;      // lsb
    //let mut register: [u8; 2] = u16::to_le_bytes(register.into());
    //i2c_master_write(cmd, register.as_mut_ptr(), 2, ACK_CHECK_EN).as_result()?;

    // restart
    i2c_master_start(cmd).as_result()?;

    // set read bit for address
    i2c_master_write_byte(cmd, (address << 1) | i2c_rw_t::I2C_MASTER_READ as u8, ACK_CHECK_EN).as_result()?;

    // read register value TODO use u32::from_le_bytes([.., ..]);
    // https://doc.rust-lang.org/std/primitive.u32.html#method.from_le_bytes
    let mut value = (0, 0);
    i2c_master_read_byte(cmd, &mut value.0, i2c_ack_type_t::I2C_MASTER_ACK);  // msb
    i2c_master_read_byte(cmd, &mut value.1, i2c_ack_type_t::I2C_MASTER_NACK); // lsb
    //let value = (((value.0 as u16) & 0xf) << 8) | (value.1 as u16);
    //let mut value: [u8; 2] = [0; 2];
    //i2c_master_read(cmd, value.as_mut_ptr(), 2, i2c_ack_type_t::I2C_MASTER_ACK).as_result()?;;
    //let value = u16::from_le_bytes(value);

    // stop
    i2c_master_stop(cmd).as_result()?;

    // send
    i2c_master_cmd_begin(port, cmd, 1000 / portTICK_RATE_MS).as_result()?;
    i2c_cmd_link_delete(cmd);

    // TODO try up top
    let value = (((value.0 as u16) & 0xf) << 8) | (value.1 as u16);
    //let value = u16::from_le_bytes(value);

    Ok(value)
}


unsafe fn write(port: i2c_port_t, address: u8, register: Address, value: u16) -> Result<(), EspError> {
    let cmd: i2c_cmd_handle_t = i2c_cmd_link_create();

    // start
    i2c_master_start(cmd).as_result()?;

    // set write bit for address
    i2c_master_write_byte(cmd, (address << 1) | i2c_rw_t::I2C_MASTER_WRITE as u8, ACK_CHECK_EN).as_result()?;

    // write value to register
    let register: u16 = register.into();
    let value: u16 = value.into();
    let mut bytes: [u8; 4] = [
        ((register >> 8) & 0xFF) as u8,
        (register & 0xFF) as u8,
        ((value >> 8) & 0xFF) as u8,
        (value & 0xFF) as u8,
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
