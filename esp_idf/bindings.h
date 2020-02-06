// - standard library ---------------------------------------------------------

#include <errno.h>
#include <math.h>
#include <sys/errno.h>
#include <netinet/in.h>


// - esp components -----------------------------------------------------------

#include <nvs_flash.h>

#include <esp_err.h>
#include <esp_event.h>
#include <esp_int_wdt.h>
#include <esp_log.h>
//#include <esp_netif.h>
#include <esp_pthread.h>
#include <esp_system.h>
#include <esp_task_wdt.h>
#include <esp_wifi.h>


// - drivers ------------------------------------------------------------------

#include <driver/adc.h>
#include <driver/dac.h>
#include <driver/gpio.h>
#include <driver/i2c.h>
#include <driver/i2s.h>
#include <driver/ledc.h>
#include <driver/spi_common.h>
#include <driver/spi_master.h>


// - lwip ---------------------------------------------------------------------

#include <lwip/err.h>
#include <lwip/ip4_addr.h>
#include <lwip/inet.h>
#include <lwip/netdb.h>
#include <lwip/netif.h>
#include <lwip/sockets.h>
#include <lwip/sys.h>

const u32_t LWIP_IPADDR_ANY = IPADDR_ANY;
const u32_t LWIP_INADDR_ANY = INADDR_ANY;


// - pthread ------------------------------------------------------------------

#include <pthread.h>


// - freertos -----------------------------------------------------------------

#include <freertos/FreeRTOS.h>
#include <freertos/event_groups.h>
#include <freertos/task.h>
