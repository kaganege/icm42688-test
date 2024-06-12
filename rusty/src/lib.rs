#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
#![no_std]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[macro_use]
extern crate alloc;

use i2c::I2C;
use icm::{Accelerometer, ICM42688};
use std::{ffi, thread};

#[macro_use]
mod std;
mod error;
mod i2c;
mod icm;

fn main() {
  unsafe {
    gpio_set_function(PICO_DEFAULT_I2C_SDA_PIN, GPIO_FUNC_I2C);
    gpio_set_function(PICO_DEFAULT_I2C_SCL_PIN, GPIO_FUNC_I2C);
  };

  let i2c = I2C::new(unsafe { i2c0_inst });
  i2c.init(800 * 1000);

  let mut icm = ICM42688::new(i2c, icm::Address::Primary);
  icm.init().unwrap();

  loop {
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

    thread::sleep_ms(500);
  }
}

#[no_mangle]
#[export_name = "main"]
pub unsafe extern "C" fn _main() -> ffi::c_int {
  stdio_init_all();
  main();

  0
}
