use cstr_core::CStr;
use cty::{c_char, c_void};

use esp_idf::bindings as idf;
use esp_idf::{AsResult, EspError, portMAX_DELAY};

use crate::logger;


// - global constants ---------------------------------------------------------

const TAG: &str = "api::wifi";

static mut WIFI_EVENT_GROUP: Option<idf::EventGroupHandle_t> = None;
const WIFI_CONNECTED_BIT: u32 = idf::BIT0;


// - exports ------------------------------------------------------------------

// TODO init(event_group: idf::EventGroupHandle_t)
pub unsafe fn init(ssid: &'static str, password: &'static str) -> Result<(), EspError> {
    log!(TAG, "initializing wifi");

    WIFI_EVENT_GROUP = Some(idf::xEventGroupCreate());

    idf::tcpip_adapter_init();

    //idf::esp_netif_init().as_result()?;
    idf::esp_event_loop_create_default().as_result()?;
    //idf::esp_netif_create_default_wifi_sta();
    let cfg: idf::wifi_init_config_t = wifi_init_config_default();
    idf::esp_wifi_init(&cfg).as_result()?;

    idf::esp_event_handler_register(idf::WIFI_EVENT, idf::ESP_EVENT_ANY_ID,
                                    Some(event_handler),
                                    core::ptr::null_mut()
    ).as_result()?;
    idf::esp_event_handler_register(idf::IP_EVENT, idf::ip_event_t::IP_EVENT_STA_GOT_IP as i32,
                                    Some(event_handler),
                                    core::ptr::null_mut()
    ).as_result()?;

    idf::esp_wifi_set_mode(idf::wifi_mode_t::WIFI_MODE_STA).as_result()?;

    // data conversion helpers
    let v32 = |s: &'static [u8], a: &mut [u8; 32]| {
        for (&x, p) in s.iter().zip(a.iter_mut()) {
            *p = x;
        }
        *a
    };
    let v64 = |s: &'static [u8], a: &mut [u8; 64]| {
        for (&x, p) in s.iter().zip(a.iter_mut()) {
            *p = x;
        }
        *a
    };
    let config_wifi_ssid:     [u8; 32] = v32(ssid.as_bytes(), &mut [0 as u8;32]);
    let config_wifi_password: [u8; 64] = v64(password.as_bytes(), &mut [0 as u8;64]);

    let mut wifi_config: idf::wifi_config_t = idf::wifi_config_t {
        sta: idf::wifi_sta_config_t {
            ssid:     config_wifi_ssid,
            password: config_wifi_password,
            ..idf::wifi_sta_config_t::default()
        },
    };
    idf::esp_wifi_set_config(idf::esp_interface_t::ESP_IF_WIFI_STA, &mut wifi_config).as_result()?;
    idf::esp_wifi_start().as_result()?;

    // // AAAAARGH: https://github.com/espressif/esp-idf/issues/3714
    idf::esp_wifi_set_ps(idf::wifi_ps_type_t::WIFI_PS_NONE);

    // debug output
    let ssid = CStr::from_ptr(config_wifi_ssid.as_ptr() as *const c_char);
    log!(TAG, "connecting to ap SSID: {}", ssid.to_str().unwrap());

    // wait for connection
    idf::xEventGroupWaitBits(WIFI_EVENT_GROUP.unwrap(), WIFI_CONNECTED_BIT, 0, 1, portMAX_DELAY);

    log!(TAG, "connected to ap SSID: {}", ssid.to_str().unwrap());

    Ok(())
}


// - event handler callback ---------------------------------------------------

#[no_mangle]
unsafe extern "C" fn event_handler(event_handler_arg: *mut c_void, event_base: idf::esp_event_base_t, event_id: i32, event_data: *mut c_void) -> () {
    const TAG: &str = "api::wifi::event_handler";

    static mut RETRY_COUNT: u32 = 0;

    if event_base == idf::WIFI_EVENT {
        if event_id == idf::wifi_event_t::WIFI_EVENT_STA_START as i32 {
            log!(TAG, "WIFI_EVENT.WIFI_EVENT_STA_START");
            idf::esp_wifi_connect();
        } else if event_id == idf::wifi_event_t::WIFI_EVENT_STA_DISCONNECTED as i32 {
            log!(TAG, "WIFI_EVENT.WIFI_EVENT_STA_DISCONNECTED");
            if RETRY_COUNT < 10 {
                log!(TAG, "Failed to connect to AP. Trying again: {}", RETRY_COUNT);
                idf::xEventGroupClearBits(WIFI_EVENT_GROUP.unwrap(), WIFI_CONNECTED_BIT);
                idf::esp_wifi_connect();
                RETRY_COUNT += 1;
            } else {
                log!(TAG, "Failed to connect to AP. Giving up.");
            }
        } else {
            log!(TAG, "WIFI_EVENT.unknown");
        }

    } else if event_base == idf::IP_EVENT {
        if event_id == idf::ip_event_t::IP_EVENT_STA_GOT_IP as i32 {
            log!(TAG, "IP_EVENT.IP_EVENT_STA_GOT_IP");
            //ip_event_got_ip_t* event = (ip_event_got_ip_t*) event_data;
            //log!(TAG, "Connected to AP with ip: " IPSTR, IP2STR(&event->ip_info.ip));
            RETRY_COUNT = 0;
            idf::xEventGroupSetBits(WIFI_EVENT_GROUP.unwrap(), WIFI_CONNECTED_BIT);
        } else {
            log!(TAG, "IP_EVENT.unknown");
        }

    } else {
        log!(TAG, "unknown.unknown");
    }
}


unsafe fn wifi_init_config_default() -> idf::wifi_init_config_t {
    idf::wifi_init_config_t {
        event_handler: Some(idf::esp_event_send),
        osi_funcs: &mut idf::g_wifi_osi_funcs,
        wpa_crypto_funcs: idf::g_wifi_default_wpa_crypto_funcs,
        static_rx_buf_num: idf::CONFIG_ESP32_WIFI_STATIC_RX_BUFFER_NUM as i32,
        dynamic_rx_buf_num: idf::CONFIG_ESP32_WIFI_DYNAMIC_RX_BUFFER_NUM as i32,
        tx_buf_type: idf::CONFIG_ESP32_WIFI_TX_BUFFER_TYPE as i32,
        static_tx_buf_num: idf::WIFI_STATIC_TX_BUFFER_NUM as i32,
        dynamic_tx_buf_num: idf::WIFI_DYNAMIC_TX_BUFFER_NUM as i32,
        csi_enable: idf::WIFI_CSI_ENABLED as i32,
        ampdu_rx_enable: idf::WIFI_AMPDU_RX_ENABLED as i32,
        ampdu_tx_enable: idf::WIFI_AMPDU_TX_ENABLED as i32,
        nvs_enable: idf::WIFI_NVS_ENABLED as i32,
        nano_enable: idf::WIFI_NANO_FORMAT_ENABLED as i32,
        tx_ba_win: idf::WIFI_DEFAULT_TX_BA_WIN as i32,
        rx_ba_win: idf::WIFI_DEFAULT_RX_BA_WIN as i32,
        wifi_task_core_id: idf::WIFI_TASK_CORE_ID as i32,
        beacon_max_len: idf::WIFI_SOFTAP_BEACON_MAX_LEN as i32,
        mgmt_sbuf_num: idf::WIFI_MGMT_SBUF_NUM as i32,
        magic: idf::WIFI_INIT_CONFIG_MAGIC as i32
    }
}
