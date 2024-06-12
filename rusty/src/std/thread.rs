#![allow(unused)]

use super::time::Duration;

pub fn sleep(duration: Duration) {
  unsafe { crate::sleep_us(duration.as_micros() as _) };
}

pub fn sleep_ms(ms: u32) {
  sleep(Duration::from_millis(ms as u64))
}
