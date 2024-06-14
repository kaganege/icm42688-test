#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
#![no_std]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[macro_use]
extern crate alloc;

// use core::ptr::addr_of_mut;
// use i2c::I2C;
use gpio::Pin;
use icm::{Accelerometer, ICM42688};
use spi::SPI;
use std::{ffi, thread};

#[macro_use]
mod std;
mod error;
mod gpio;
mod i2c;
mod icm;
mod spi;

fn main() {
  unsafe {
    // gpio_set_function(PICO_DEFAULT_I2C_SDA_PIN, GPIO_FUNC_I2C);
    // gpio_set_function(PICO_DEFAULT_I2C_SCL_PIN, GPIO_FUNC_I2C);
    // gpio_pull_up(PICO_DEFAULT_I2C_SDA_PIN);
    // gpio_pull_up(PICO_DEFAULT_I2C_SCL_PIN);
    gpio_set_function(PICO_DEFAULT_SPI_RX_PIN, GPIO_FUNC_SPI);
    gpio_set_function(PICO_DEFAULT_SPI_SCK_PIN, GPIO_FUNC_SPI);
    gpio_set_function(PICO_DEFAULT_SPI_TX_PIN, GPIO_FUNC_SPI);
    gpio_set_function(PICO_DEFAULT_SPI_CSN_PIN, GPIO_FUNC_SPI);
  };

  thread::sleep_ms(10_000);

  // let i2c = I2C::new(unsafe { addr_of_mut!(i2c0_inst) });
  // i2c.init(600_000);
  // i2c.init(100_000);

  let mut spi = SPI::default();
  // let mut spi = SPI::with_chip_select(Default::default(), gpio::Pin::new_init(PICO_DEFAULT_SPI_CSN_PIN));
  spi.init(1_000_000);

  // let mut icm = ICM42688::with_i2c(i2c, icm::Address::Primary);
  let mut icm = ICM42688::with_spi(spi);
  icm.init().unwrap();

  thread::sleep_ms(5000);

  loop {
    println!("I'm alive!");

    icm
      .temperature()
      .inspect(|temperature| {
        dbg!(temperature);
      })
      .ok();

    icm
      .accel_norm()
      .inspect(|accel_data| {
        dbg!(accel_data);
      })
      .ok();

    icm
      .gyro_norm()
      .inspect(|gyro_data| {
        dbg!(gyro_data);
      })
      .ok();

    // let scan_result = i2c.scan();
    // dbg!(scan_result);

    debugln!("Waiting 5 seconds");

    thread::sleep_ms(5_000);
  }
}

#[no_mangle]
#[export_name = "main"]
pub unsafe extern "C" fn _main() -> ffi::c_int {
  stdio_init_all();
  main();

  0
}
