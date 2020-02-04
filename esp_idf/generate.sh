#!/usr/bin/env zsh

# - configuration -------------------------------------------------------------

XTENSA_ROOT=/mnt/flowdsp/deps/versions/rust-xtensa
IDF_PATH=/mnt/flowdsp/deps/esp-idf


# - paths ---------------------------------------------------------------------

LLVM_ROOT=${XTENSA_ROOT}/llvm_build

ENVPATH="~/.cargo/bin"
ENVPATH="${ENVPATH}:${LLVM_ROOT}/bin"


# - includes ------------------------------------------------------------------

IDF_COMPONENTS=${IDF_PATH}/components

INCLUDES="${INCLUDES} -I${LLVM_ROOT}/lib/clang/6.0.1/include"

INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/driver/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/esp32/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/esp_common/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/esp_event/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/esp_netif/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/esp_ringbuf/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/esp_rom/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/esp_wifi/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/freertos/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/heap/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/log/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/lwip/include/apps"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/lwip/include/apps/sntp"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/lwip/lwip/src/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/lwip/port/esp32/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/newlib/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/newlib/platform_include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/nvs_flash/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/pthread/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/soc/esp32/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/soc/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/spi_flash/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/tcpip_adapter/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/vfs/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/xtensa/esp32/include"
INCLUDES="${INCLUDES} -I${IDF_COMPONENTS}/xtensa/include"

INCLUDES="${INCLUDES} -I../build/config"


# - generate bindings ---------------------------------------------------------

ENVIRONMENT="${ENVIRONMENT} PATH=${ENVPATH}"
ENVIRONMENT="${ENVIRONMENT} LLVM_CONFIG_PATH=${LLVM_ROOT}/bin/llvm-config"
ENVIRONMENT="${ENVIRONMENT} LIBCLANG_PATH=${LLVM_ROOT}/lib"

CLANG_FLAGS="-nostdinc -target xtensa"
CLANG_WARN="-Wno-macro-redefined -Wno-unknown-attributes"
CLANG_ARGS="${CLANG_FLAGS} ${CLANG_WARN} ${INCLUDES}"

BINDGEN_FLAGS="--no-doc-comments"
BINDGEN_FLAGS="${BINDGEN_FLAGS} --use-core --ctypes-prefix=std::os::raw"
BINDGEN_FLAGS="${BINDGEN_FLAGS} --builtins"
BINDGEN_FLAGS="${BINDGEN_FLAGS} --conservative-inline-namespaces"
BINDGEN_FLAGS="${BINDGEN_FLAGS} --distrust-clang-mangling"
BINDGEN_FLAGS="${BINDGEN_FLAGS} --enable-function-attribute-detection"
BINDGEN_FLAGS="${BINDGEN_FLAGS} --generate-inline-functions"
BINDGEN_FLAGS="${BINDGEN_FLAGS} --disable-name-namespacing"
BINDGEN_FLAGS="${BINDGEN_FLAGS} --disable-nested-struct-naming"
BINDGEN_FLAGS="${BINDGEN_FLAGS} --generate-block"
BINDGEN_FLAGS="${BINDGEN_FLAGS} --impl-debug"
BINDGEN_FLAGS="${BINDGEN_FLAGS} --impl-partialeq"
BINDGEN_FLAGS="${BINDGEN_FLAGS} --with-derive-default"
BINDGEN_FLAGS="${BINDGEN_FLAGS} --with-derive-eq"
BINDGEN_FLAGS="${BINDGEN_FLAGS} --with-derive-partialeq"
BINDGEN_FLAGS="${BINDGEN_FLAGS} --use-array-pointers-in-arguments"
BINDGEN_FLAGS="${BINDGEN_FLAGS} --no-layout-tests"
BINDGEN_FLAGS="${BINDGEN_FLAGS} --no-prepend-enum-name"

BINDGEN_OPTIONS="--output src/bindings.rs"
BINDGEN_OPTIONS="${BINDGEN_OPTIONS} --rust-target 1.36"
#BINDGEN_OPTIONS="${BINDGEN_OPTIONS} --default-alias-style new_type"
BINDGEN_OPTIONS="${BINDGEN_OPTIONS} --default-enum-style rust"
BINDGEN_OPTIONS="${BINDGEN_OPTIONS} --blacklist-item 'CONFIG_FIRMWARE_.+'"
#BINDGEN_OPTIONS="${BINDGEN_OPTIONS} --whitelist-function '(esp|ESP)_.+'"
#BINDGEN_OPTIONS="${BINDGEN_OPTIONS} --whitelist-function '(gpio|GPIO)_.+'"
#BINDGEN_OPTIONS="${BINDGEN_OPTIONS} --whitelist-function '(i2c_|I2C_).+'"
#BINDGEN_OPTIONS="${BINDGEN_OPTIONS} --whitelist-function '(i2s_|I2S_).+'"
#BINDGEN_OPTIONS="${BINDGEN_OPTIONS} --whitelist-function '(lwip|LWIP)_.+'"
#BINDGEN_OPTIONS="${BINDGEN_OPTIONS} --whitelist-function '(spi_|spicommon_).+'"
#BINDGEN_OPTIONS="${BINDGEN_OPTIONS} --whitelist-function '(wifi|WIFI)_.+'"
#BINDGEN_OPTIONS="${BINDGEN_OPTIONS} --whitelist-function 'ip(4|6)addr_.+'
#BINDGEN_OPTIONS="${BINDGEN_OPTIONS} --whitelist-function 'nvs_flash_.+'"
#BINDGEN_OPTIONS="${BINDGEN_OPTIONS} --whitelist-function 'tcpip_.+'"
#BINDGEN_OPTIONS="${BINDGEN_OPTIONS} --whitelist-function 'vTaskDelay'"
BINDGEN_OPTIONS="${BINDGEN_OPTIONS} --bitfield-enum '(i2s|I2S)_.+'"

BINDGEN_HEADER="bindings.h"

BINDGEN="${ENVIRONMENT} bindgen ${BINDGEN_FLAGS} ${BINDGEN_OPTIONS} ${BINDGEN_HEADER} -- ${CLANG_ARGS}"

eval ${BINDGEN}
