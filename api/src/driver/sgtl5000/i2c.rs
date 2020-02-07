use num_enum::IntoPrimitive;

use esp_idf::{AsResult, EspError, portMAX_DELAY};
use esp_idf::bindings::{i2c_port_t};


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

const ACK_CHECK_EN: u8  = 0x1; // I2C master will check ack from slave
const ACK_CHECK_DIS: u8 = 0x0; // I2C master will not check ack from slave
const ACK_VAL: u8       = 0x0; // I2C ack value
const NACK_VAL: u8      = 0x1; // I2C nack value


fn foo() {
    let plonk: u16 = Address::CHIP_ID.into();
}

fn read(port: i2c_port_t) -> Result<u16, EspError> {

    Ok(23)
}
