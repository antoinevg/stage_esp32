use cstr_core::{CStr, c_char};
use cty::{c_void, c_int};

use esp_idf::bindings as idf;
use esp_idf::{AsResult, EspError, errno};

use crate::logger;

// - global constants ---------------------------------------------------------

const TAG: &str = "lwip";


// - types --------------------------------------------------------------------

type Socket = c_int;


// - exports ------------------------------------------------------------------

pub unsafe fn socket(domain: u32, socket_type: u32, protocol: u32) -> Result<Socket, EspError> {
    let socket: c_int = idf::lwip_socket(domain as c_int, socket_type as c_int, protocol as c_int);
    if socket < 0 {
        log!(TAG, "error Unable to create socket: errno {:?}", errno());
        return Err(EspError(errno() as idf::esp_err_t)); // TODO we need an errno field on EspError
    }

    Ok(socket)
}


pub unsafe fn bind(socket: Socket, address: u32, family: u32, port: u16) -> Result<(), EspError> {
    let mut dest_addr: idf::sockaddr_in = idf::sockaddr_in::default();
    dest_addr.sin_addr.s_addr = idf::lwip_htonl(address);
    dest_addr.sin_family = family as idf::sa_family_t;
    dest_addr.sin_port = idf::lwip_htons(port);

    let err = idf::lwip_bind(socket,
                             core::mem::transmute::<&idf::sockaddr_in,
                                                    &idf::sockaddr>(&dest_addr),
                             core::mem::size_of::<idf::sockaddr_in>() as u32);
    if err < 0 {
        log!(TAG, "error: Socket unable to bind: errno {:?}", errno());
        return Err(EspError(errno() as idf::esp_err_t)); // TODO we need an errno field on EspError
    }

    log!(TAG, "socket bound to port: {}", port);

    Ok(())
}


pub unsafe fn recvfrom(socket: Socket) -> Result<([u8; 128], usize), EspError> {
    let mut rx_buffer:[u8; 128] = [0; 128]; // TODO supply receive buffer in arguments

    let mut source_addr: idf::sockaddr_in = idf::sockaddr_in::default();
    let mut socklen: idf::socklen_t = core::mem::size_of::<idf::sockaddr_in>() as idf::socklen_t;

    let len = idf::lwip_recvfrom(socket,
                                 rx_buffer.as_mut_ptr() as *mut c_void,
                                 rx_buffer.len(),
                                 0,
                                 core::mem::transmute::<&mut idf::sockaddr_in,
                                                        &mut idf::sockaddr>(&mut source_addr),
                                 &mut socklen);
    if len < 0 {
        log!(TAG, "recvfrom error: errno {:?}", errno());
        return Err(EspError(errno() as idf::esp_err_t)); // TODO we need an errno field on EspError
    }

    // log source ip_address
    /*let addr: idf::ip4_addr = idf::ip4_addr {
        addr: source_addr.sin_addr.s_addr
    };
    let mut ip_address:[u8; 128] = [0; 128];
    idf::ip4addr_ntoa_r(&addr as *const idf::ip4_addr,
                        ip_address.as_mut_ptr() as *mut i8,
                        (ip_address.len() - 1) as i32);
    let ip_address  = CStr::from_ptr(ip_address.as_ptr() as *const c_char).to_str().unwrap();
    log!(TAG, "received {} bytes from {}", len, ip_address);*/

    Ok((rx_buffer, len as usize))
}

static mut ERROR_COUNT: usize = 0;

pub unsafe fn sendto(socket: Socket, buffer: &[u8], address: u32, family: u32, port: u16) -> Result<isize, EspError> {
    let mut dest_addr: idf::sockaddr_in = idf::sockaddr_in::default();
    dest_addr.sin_addr.s_addr = address;
    dest_addr.sin_family = family as idf::sa_family_t;
    dest_addr.sin_port = idf::lwip_htons(port);

    let dest_addr = core::mem::transmute::<&idf::sockaddr_in,
                                           &idf::sockaddr>(&dest_addr);
    let socklen: idf::socklen_t = core::mem::size_of::<idf::sockaddr>() as idf::socklen_t;

    let bytes_sent = idf::lwip_sendto(socket,
                                      buffer.as_ptr() as *const c_void,
                                      buffer.len(),
                                      0,
                                      dest_addr,
                                      socklen);


    if bytes_sent < 1 {
        ERROR_COUNT += 1;
        log!(TAG, "sendto error #{}: {} (sent {} bytes)", ERROR_COUNT, errno(), bytes_sent);
        let error_number = errno() as u32;
        if  error_number == idf::ENOMEM {
            // see: https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-guides/lwip.html#limitations
            let mut counter = 0;
            loop  {
                counter += 1;
                let bytes_sent = idf::lwip_sendto(socket,
                                                  buffer.as_ptr() as *const c_void,
                                                  buffer.len(),
                                                  0,
                                                  dest_addr,
                                                  socklen);
                if bytes_sent > 0 {
                    log!(TAG, "sendto succeeded after {} retries", counter);
                    return Ok(bytes_sent);
                }
                idf::vTaskDelay(1);
            }
        }
    }

    // TODO handle rest of errors

    Ok(bytes_sent)
}
