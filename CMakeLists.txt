# Generated Cmake Pico project file

cmake_minimum_required(VERSION 3.13)

set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Initialise pico_sdk from installed location
# (note this can come from environment, CMake cache etc)

# == DO NEVER EDIT THE NEXT LINES for Raspberry Pi Pico VS Code Extension to work ==
if(WIN32)
   set(USERHOME $ENV{USERPROFILE})
else()
    set(USERHOME $ENV{HOME})
endif()
set(PICO_SDK_PATH ${USERHOME}/.pico-sdk/sdk/1.5.1)
set(PICO_TOOLCHAIN_PATH ${USERHOME}/.pico-sdk/toolchain/13_2_Rel1)
if(WIN32)
    set(pico-sdk-tools_DIR ${USERHOME}/.pico-sdk/tools/1.5.1)
    include(${pico-sdk-tools_DIR}/pico-sdk-tools-config.cmake)
    include(${pico-sdk-tools_DIR}/pico-sdk-tools-config-version.cmake)
endif()
# ====================================================================================

set(PICO_BOARD pico_w CACHE STRING "Board type")

set(CARGO_ARGS "build")

if(NOT CMAKE_BUILD_TYPE)
  set(LIB_BUILD_TYPE "debug")
elseif(${CMAKE_BUILD_TYPE} STREQUAL "Release")
  set(LIB_BUILD_TYPE "release")
else()
  set(LIB_BUILD_TYPE "debug")
endif()

if(${LIB_BUILD_TYPE} STREQUAL "release")
  list(APPEND CARGO_ARGS "--release")
endif()

set(LIB_FILE "${CMAKE_SOURCE_DIR}/rusty/target/thumbv6m-none-eabi/${LIB_BUILD_TYPE}/librusty.a")

# Pull in Raspberry Pi Pico SDK (must be before project)
include(pico_sdk_import.cmake)

if (PICO_SDK_VERSION_STRING VERSION_LESS "1.4.0")
  message(FATAL_ERROR "Raspberry Pi Pico SDK version 1.4.0 (or later) required. Your version is ${PICO_SDK_VERSION_STRING}")
endif()

project(icm42688-test C CXX ASM)

# Initialise the Raspberry Pi Pico SDK
pico_sdk_init()

# Add executable. Default name is the project name, version 0.1

add_executable(icm42688-test main.cpp )

pico_set_program_name(icm42688-test "icm42688-test")
pico_set_program_version(icm42688-test "0.1")

# Modify the below lines to enable/disable output over UART/USB

pico_enable_stdio_uart(icm42688-test 0)


pico_enable_stdio_usb(icm42688-test 1)

# Add the standard library to the build
target_link_libraries(icm42688-test
  pico_stdlib
)

# Add the standard include files to the build
target_include_directories(icm42688-test PRIVATE
  ${CMAKE_CURRENT_LIST_DIR}
  ${CMAKE_CURRENT_LIST_DIR}/.. # for our common lwipopts or any other standard includes, if required
)

# Add Rust project
add_custom_command(
  OUTPUT ${LIB_FILE}
  COMMAND cargo ARGS ${CARGO_ARGS}
  WORKING_DIRECTORY ${CMAKE_CURRENT_LIST_DIR}/rusty
  COMMENT "running cargo"
)

# Create rust project
add_custom_target(rusty ALL DEPENDS ${LIB_FILE})

# Create dependency
add_dependencies(icm42688-test rusty)

# Add any user requested libraries
target_link_libraries(icm42688-test
  pico_cyw43_arch_none
  hardware_timer
  hardware_i2c
  ${LIB_FILE}
)

pico_add_extra_outputs(icm42688-test)
