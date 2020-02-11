use esp_idf::bindings::{
    gpio_num_t,
    i2c_port_t,
};


// - types --------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Pins {
    pub scl: gpio_num_t,
    pub sda: gpio_num_t,
}

impl Pins {
    pub fn new() -> Pins {
        Pins {
            scl:  gpio_num_t::GPIO_NUM_23,
            sda:  gpio_num_t::GPIO_NUM_18,
        }
    }
}
