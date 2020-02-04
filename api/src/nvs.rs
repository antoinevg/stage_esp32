use esp_idf::bindings as idf;
use esp_idf::{AsResult, EspError};

use crate::logger;


// - global constants ---------------------------------------------------------

const TAG: &str = "nvs";


// - exports ------------------------------------------------------------------

pub unsafe fn flash_init() -> Result<(), EspError> {
    match idf::nvs_flash_init().as_result() {
        Ok(()) => (),
        Err(EspError(e)) => {
            if e == idf::ESP_ERR_NVS_NO_FREE_PAGES as i32 || e == idf::ESP_ERR_NVS_NEW_VERSION_FOUND as i32 {
                log!(TAG, "erasing flash: {:?}", e);
                idf::nvs_flash_erase().as_result()?;
                idf::nvs_flash_init().as_result()?;
            }
        }
    }

    Ok(())
}
