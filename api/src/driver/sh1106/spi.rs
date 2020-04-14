use core::convert::TryFrom;

use esp_idf::{AsResult, EspError, portMAX_DELAY, portTICK_RATE_MS};
use esp_idf::bindings::{
    gpio_config_t,
    gpio_int_type_t,
    gpio_mode_t,
    gpio_num_t,
    gpio_pulldown_t,
    gpio_pullup_t,
    spi_bus_config_t,
    spi_device_interface_config_t,
    spi_device_handle_t,
    spi_device_t,
    spi_host_device_t,
    spi_transaction_t,
};
use esp_idf::bindings::{
    gpio_config,
    gpio_set_level,
    spi_bus_initialize,
    spi_bus_add_device,
    spi_device_polling_transmit,
};
use esp_idf::bindings as idf;

use cty::{uint8_t, c_void};
use embedded_graphics::{
    fonts::Font6x8,
    icoord,
    prelude::*,
    primitives::{Circle, Line},
};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::blinky;
use crate::logger;


// - global constants ---------------------------------------------------------

const TAG: &str = "api::driver::sh1106::spi";


// - types --------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Pins {
    pub csel: gpio_num_t,
    pub sclk: gpio_num_t,
    pub mosi: gpio_num_t,
    pub dc: gpio_num_t,
}

impl Pins {
    pub fn new() -> Pins {
        Pins {
            csel:  gpio_num_t::GPIO_NUM_5,
            sclk:  gpio_num_t::GPIO_NUM_18,
            mosi:  gpio_num_t::GPIO_NUM_23,
            dc:    gpio_num_t::GPIO_NUM_19,
        }
    }
}

#[derive(Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Mode {
    Command = 0,
    Data    = 1
}

enum Power {
    EXTERNALVCC          = 0x1,
    SWITCHCAPVCC         = 0x2,
}


// - initialization -----------------------------------------------------------

pub unsafe fn init(device: spi_host_device_t, pins: Pins) -> Result<(spi_device_handle_t), EspError> {
    // also see: https://github.com/kcl93/adafruit_esp-idf_sh1106/blob/master/Adafruit_SH1106.cpp

    log!(TAG, "configure spi pins for display peripheral: {:?}", pins);

    // configure pins as outputs: csel, d/c
    let config = gpio_config_t {
        pin_bit_mask: (0x1 << (pins.csel as u32)) | (0x1 << (pins.dc as u32)),
            //| (0x1 << (pins.sclk as u32))
            //| (0x1 << (pins.mosi as u32)),
        mode: gpio_mode_t::GPIO_MODE_OUTPUT,
        pull_down_en: gpio_pulldown_t::GPIO_PULLDOWN_DISABLE,
        pull_up_en: gpio_pullup_t::GPIO_PULLUP_DISABLE,
        intr_type: gpio_int_type_t::GPIO_INTR_DISABLE,
    };
    gpio_config(&config).as_result()?;

    // initialize spi bus
    log!(TAG, "initialize spi host device: {:?}", device);
    let config = spi_bus_config_t {
        sclk_io_num: pins.sclk as i32,
        mosi_io_num: pins.mosi as i32,
        miso_io_num: gpio_num_t::GPIO_NUM_NC as i32,
        quadwp_io_num: gpio_num_t::GPIO_NUM_NC as i32,
        quadhd_io_num: gpio_num_t::GPIO_NUM_NC as i32,
        max_transfer_sz: 128,
        flags: 0,
        intr_flags: 0,
    };
    let dma_channel = 1;
    spi_bus_initialize(device, &config, dma_channel);

    // add display to the spi bus
    log!(TAG, "add display to spi bus");
    let config = spi_device_interface_config_t {
        clock_speed_hz: 4_000_000,           // 4MHz
        mode: 0,                             // SPI mode 0
        spics_io_num: pins.csel as i32,
        queue_size: 2,                       // queue 2 transactions
        pre_cb: Some(pre_transfer_callback), // callback to handle d/c line
        ..spi_device_interface_config_t::default()
    };
    let mut handle = core::ptr::null_mut();
    spi_bus_add_device(device, &config, &mut handle); // TODO check returned handle for null pointer

    // pre-transfer callback
    extern "C" fn pre_transfer_callback(transaction: *mut idf::spi_transaction_t) -> ()  {
        const TAG: &str = "api::driver::sh1106::spi::pre_transfer_callback";

        // TODO super shitty, give it a struct please
        let transaction: spi_transaction_t = unsafe { *transaction };
        let user: *mut core::ffi::c_void = transaction.user;
        let user: *const uint8_t = user as *const uint8_t;
        let user: &[u8] = unsafe { core::slice::from_raw_parts(user, 2) };
        let gpio_dc: gpio_num_t = unsafe { core::mem::transmute::<i32, gpio_num_t>(user[0] as i32) };
        let mode: Mode = Mode::try_from(user[1]).unwrap();
        //log!(TAG, "gpio_dc: {:?}  mode: {:?} ({})", gpio_dc, mode, user[1]);

        let mode: u8 = mode.into();
        unsafe { gpio_set_level(gpio_dc, mode as u32) };
    }

    Ok(handle)
}


pub unsafe fn configure(handle: spi_device_handle_t, gpio_dc: gpio_num_t) -> Result<(), EspError> {
    let delay = (0.001 * 168_000_000.) as u32;

    log!(TAG, "configuring sh1106 oled display with handle:{:?} dc:{:?}", handle, gpio_dc);

    let command = |bytes: &[u8]| -> Result<(), EspError> {
        transmit(handle, gpio_dc, bytes, Mode::Command)
    };

    //let vcc = Power::EXTERNALVCC;
    let vcc = Power::SWITCHCAPVCC;

    // configure display
    command(&[Register::DISPLAYOFF.into()])?;           // 0xAE
    command(&[Register::SETLOWCOLUMN.into()])?;         // 0x02
    command(&[Register::SETHIGHCOLUMN.into()])?;        // 0x10
    command(&[Register::SETSTARTLINE.into()])?;         // 0x40
    command(&[Register::SETCONTRAST.into()])?;          // 0x81
    match vcc {
        Power::EXTERNALVCC  => command(&[0x9f]),
        Power::SWITCHCAPVCC => command(&[0xcf]),
    }?;
    command(&[Register::SEGREMAP.into()])?;             // 0xA0
    command(&[Register::COMSCANINC.into()])?;           // 0xC0
    command(&[Register::NORMALDISPLAY.into()])?;        // 0xA6
    command(&[Register::SETMULTIPLEX.into()])?;         // 0xA8
    command(&[0x3F])?;                              // 1/64 duty
    command(&[Register::SETDISPLAYOFFSET.into()])?;     // 0xD3
    command(&[0x00])?;                              // no offset
    command(&[Register::SETDISPLAYCLOCKDIV.into()])?;   // 0xD5
    command(&[0x80])?;                              // 100 Frames/Sec
    command(&[Register::SETPRECHARGE.into()])?;         // 0xD9
    match vcc {
        Power::EXTERNALVCC  => command(&[0x22]),
        Power::SWITCHCAPVCC => command(&[0xf1]),    // 15 Clocks & Discharge as 1 Clock
    }?;
    command(&[Register::CHARGEPUMP.into()])?;           // 0x8D
    match vcc {
        Power::EXTERNALVCC  => command(&[0x10]),
        Power::SWITCHCAPVCC => command(&[0x14]),
    }?;
    command(&[Register::SETCOMPINS.into()])?;           // 0xDA
    command(&[0x12])?;
    command(&[Register::SETVCOMDETECT.into()])?;        // 0xDB
    command(&[0x40])?;
    command(&[Register::MEMORYMODE.into()])?;           // 0x20
    command(&[0x02])?;                        // 0x0 act like ks0108, 0x2 ???
    command(&[Register::DISPLAYALLON_RESUME.into()])?;  // 0xA4
    command(&[Register::NORMALDISPLAY.into()])?;        // 0xA6

    // turn on display
    command(&[Register::DISPLAYON.into()])?;

    // allocate a page_buffer
    log!(TAG, "allocating memory for page buffer");
    #[allow(non_upper_case_globals)] const width: usize = 128;
    let mut page_buffer: [u8; width] = [0x00; width];

    // blank display
    for page in 0usize..8 {
        let page_address = (0xb0 + page) as u8;
        command(&[page_address])?;                   // set page address
        command(&[Register::SETLOWCOLUMN.into()])?;  // set lower column address
        command(&[Register::SETHIGHCOLUMN.into()])?; // set higher column address
        command(&[Register::SETSTARTLINE.into()])?;
        transmit(handle, gpio_dc, &page_buffer, Mode::Data)?;       // write data for page
    }

    // generate data for a test pattern
    log!(TAG, "generate test pattern data");
    for x in 0..width {
        let byte = if x % 8 == 0 { 255 } else { 1 };
        page_buffer[x] = byte;
    }

    // blit test pattern to the display
    log!(TAG, "display test pattern");
    for page in 0usize..8 {
        let page_address = (0xb0 + page) as u8;
        command(&[page_address])?;                   // set page address
        command(&[Register::SETLOWCOLUMN.into()])?;  // set lower column address
        command(&[Register::SETHIGHCOLUMN.into()])?; // set higher column address
        command(&[Register::SETSTARTLINE.into()])?;
        transmit(handle, gpio_dc, &page_buffer, Mode::Data)?;       // write data for page
    }

    idf::vTaskDelay(100);

    Ok(())
}


pub unsafe fn transmit(handle: spi_device_handle_t, gpio_dc: gpio_num_t, bytes: &[u8], mode: Mode) -> Result<(), EspError> {
    //log!(TAG, "transmit dc:{:?} handle:{:?} bytes:{:x?} mode:{:?}", gpio_dc, handle, bytes, mode);
    let mut transaction = spi_transaction_t {
        length: bytes.len() * 8, // spi transaction length is measure in bits
        __bindgen_anon_1: idf::spi_transaction_t__bindgen_ty_1 {
            tx_buffer: bytes.as_ptr() as *const c_void,
        },
        ..spi_transaction_t::default()
    };

    // TODO super shitty, give it a struct please
    let mut user: [u8; 2] = [gpio_dc as u8, mode.into()];
    transaction.user = user.as_mut_ptr() as *mut c_void;

    spi_device_polling_transmit(handle, &mut transaction).as_result()?;

    Ok(())
}


// - codec register addresses -------------------------------------------------

#[allow(non_camel_case_types)]
#[derive(IntoPrimitive)]
#[repr(u8)]
pub enum Register {
    SETCONTRAST          = 0x81,
    DISPLAYALLON_RESUME  = 0xA4,
    DISPLAYALLON         = 0xA5,
    NORMALDISPLAY        = 0xA6,
    INVERTDISPLAY        = 0xA7,
    DISPLAYOFF           = 0xAE,
    DISPLAYON            = 0xAF,

    SETDISPLAYOFFSET     = 0xD3,
    SETCOMPINS           = 0xDA,

    SETVCOMDETECT        = 0xDB,

    SETDISPLAYCLOCKDIV   = 0xD5,
    SETPRECHARGE         = 0xD9,

    SETMULTIPLEX         = 0xA8,

    SETLOWCOLUMN         = 0x02,
    SETHIGHCOLUMN        = 0x10,
    SETSTARTLINE         = 0x40,

    SETPAGEADDRESS       = 0xB0,

    MEMORYMODE           = 0x20,
    COLUMNADDR           = 0x21,
    PAGEADDR             = 0x22,

    COMSCANINC           = 0xC0,
    COMSCANDEC           = 0xC8,

    SEGREMAP             = 0xA0,

    CHARGEPUMP           = 0x8D,
}
