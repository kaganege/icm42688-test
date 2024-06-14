#![allow(unused)]

use crate::{gpio_init, gpio_set_pulls, sio_hw_t};

const SIO_BASE: u32 = 0xd0000000;
const SIO_PTR: *mut sio_hw_t = SIO_BASE as _;

pub enum Direction {
  IN,
  OUT,
}

/// Set a number of GPIOs to output
pub unsafe fn gpio_set_dir_out_masked(mask: u32) {
  (*SIO_PTR).gpio_oe_set = mask;
}

/// Set a number of GPIOs to input
pub unsafe fn gpio_set_dir_in_masked(mask: u32) {
  (*SIO_PTR).gpio_oe_clr = mask;
}

/// Set a single GPIO direction
pub unsafe fn gpio_set_dir(gpio: u32, dir: Direction) {
  let mask = 1u32 << gpio;

  match dir {
    Direction::IN => gpio_set_dir_in_masked(mask),
    Direction::OUT => gpio_set_dir_out_masked(mask),
  };
}

/// Check if a specific GPIO direction is OUT
pub unsafe fn gpio_is_dir_out(gpio: u32) -> bool {
  ((*SIO_PTR).gpio_oe & (1 << gpio)) > 0
}

/// Get a specific GPIO direction
pub unsafe fn gpio_get_dir(gpio: u32) -> Direction {
  if gpio_is_dir_out(gpio) {
    Direction::OUT
  } else {
    Direction::IN
  }
}

/// Drive high every GPIO appearing in mask
pub unsafe fn gpio_set_mask(mask: u32) {
  (*SIO_PTR).gpio_set = mask;
}

/// Drive low every GPIO appearing in mask
pub unsafe fn gpio_clr_mask(mask: u32) {
  (*SIO_PTR).gpio_clr = mask;
}

/// Drive a single GPIO high/low
pub unsafe fn gpio_put(gpio: u32, value: bool) {
  let mask = 1u32 << gpio;

  if value {
    gpio_set_mask(mask);
  } else {
    gpio_clr_mask(mask);
  }
}

/// Drive all pins simultaneously
pub unsafe fn gpio_put_all(value: u32) {
  (*SIO_PTR).gpio_out = value;
}

/// Determine whether a GPIO is currently driven high or low
///
/// This function returns the high/low output level most recently assigned to a
/// GPIO via [gpio_put](gpio_put) or similar. This is the value that is
/// presented outward to the IO muxing, *not* the input level back from the pad
/// (which can be read using [gpio_put](gpio_put)).
///
/// To avoid races, this function must not be used for read-modify-write
/// sequences when driving GPIOs -- instead functions like [gpio_put](gpio_put)
/// should be used to atomically update GPIOs. This accessor is intended for
/// debug use only.
pub unsafe fn gpio_get_out_level(gpio: u32) -> bool {
  ((*SIO_PTR).gpio_out & (1 << gpio)) > 0
}

pub unsafe fn gpio_pull_up(gpio: u32) {
  gpio_set_pulls(gpio, true, false)
}

pub unsafe fn gpio_pull_down(gpio: u32) {
  gpio_set_pulls(gpio, false, true)
}

#[derive(Debug)]
pub struct Pin(u32);
impl Pin {
  pub fn new(gpio: u32) -> Self {
    Self(gpio)
  }

  pub fn init(&self) {
    unsafe { gpio_init(self.0) }
  }

  pub fn new_init(gpio: u32) -> Self {
    let pin = Self::new(gpio);
    pin.init();

    pin
  }

  pub fn init_with_direction(&self, dir: Direction) {
    self.init();
    self.set_direction(dir);
  }

  pub fn set_direction(&self, dir: Direction) {
    unsafe { gpio_set_dir(self.0, dir) }
  }

  pub fn get_direction(&self) -> Direction {
    unsafe { gpio_get_dir(self.0) }
  }

  pub fn put(&self, value: bool) {
    unsafe { gpio_put(self.0, value) }
  }

  pub fn get(&self) -> bool {
    unsafe { gpio_get_out_level(self.0) }
  }
}
