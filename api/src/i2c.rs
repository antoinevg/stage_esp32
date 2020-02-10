use esp_idf::bindings::{
    gpio_num_t,
    i2c_port_t,
};


// - types --------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Pins {
    pub scl: u8,
    pub sda: u8,
}

impl Pins {
    pub fn new() -> Pins {
        Pins {
            scl:  gpio_num_t::GPIO_NUM_23  as u8,
            sda:  gpio_num_t::GPIO_NUM_18  as u8,
        }
    }
}
