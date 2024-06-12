#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
#![no_std]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[macro_use]
extern crate alloc;

use core::ptr::addr_of_mut;
use i2c::I2C;
use icm::{Accelerometer, ICM42688};
use std::{ffi, thread};

#[macro_use]
mod std;
mod error;
mod i2c;
mod icm;

pub(crate) unsafe fn gpio_pull_up(gpio: u32) {
  gpio_set_pulls(gpio, true, false)
}

pub(crate) unsafe fn gpio_pull_down(gpio: u32) {
  gpio_set_pulls(gpio, false, true)
}

fn main() {
  unsafe {
    gpio_set_function(PICO_DEFAULT_I2C_SDA_PIN, GPIO_FUNC_I2C);
    gpio_set_function(PICO_DEFAULT_I2C_SCL_PIN, GPIO_FUNC_I2C);
    gpio_pull_up(PICO_DEFAULT_I2C_SDA_PIN);
    gpio_pull_up(PICO_DEFAULT_I2C_SCL_PIN);
  };

  let i2c = I2C::new(unsafe { addr_of_mut!(i2c0_inst) });
  i2c.init(100 * 1000);

  // let mut icm = ICM42688::new(i2c, icm::Address::Primary);
  // icm.init().unwrap();

  thread::sleep_ms(5000);

  loop {
    // println!("I'm alive!");

    // icm
    //   .temperature()
    //   .inspect(|temperature| {
    //     dbg!(temperature);
    //   })
    //   .ok();

    // icm
    //   .accel_norm()
    //   .inspect(|accel_data| {
    //     dbg!(accel_data);
    //   })
    //   .ok();

    // icm
    //   .gyro_norm()
    //   .inspect(|gyro_data| {
    //     dbg!(gyro_data);
    //   })
    //   .ok();

    let scan_result = i2c.scan();
    dbg!(scan_result);

    debugln!("Waiting 15 seconds");

    thread::sleep_ms(15_000);
  }
}

#[no_mangle]
#[export_name = "main"]
pub unsafe extern "C" fn _main() -> ffi::c_int {
  stdio_init_all();
  main();

  0
}
