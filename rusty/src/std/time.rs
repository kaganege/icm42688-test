#![allow(unused)]

pub use crate::absolute_time_t as AbsoluteTime;
use core::fmt;
use core::ops::{Add, AddAssign, Sub, SubAssign};
pub use core::time::*;

pub trait ToAbsoluteTime {
  fn to_absolute_time(&self) -> AbsoluteTime;
}

impl ToAbsoluteTime for Duration {
  fn to_absolute_time(&self) -> AbsoluteTime {
    AbsoluteTime {
      _private_us_since_boot: self.min(&Duration::from_micros(u64::MAX)).as_micros() as _,
    }
  }
}

pub struct Instant(Duration);

impl Instant {
  #[must_use]
  pub fn now() -> Instant {
    Instant(Duration::from_micros(unsafe { crate::time_us_64() }))
  }

  pub fn duration_since(&self, earlier: Instant) -> Duration {
    self.checked_duration_since(earlier).unwrap_or_default()
  }

  pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
    self.0.checked_sub(earlier.0)
  }

  pub fn checked_add(&self, duration: Duration) -> Option<Instant> {
    self.0.checked_add(duration).map(Instant)
  }

  pub fn checked_sub(&self, duration: Duration) -> Option<Instant> {
    self.0.checked_sub(duration).map(Instant)
  }
}

impl Add<Duration> for Instant {
  type Output = Instant;

  fn add(self, other: Duration) -> Instant {
    self
      .checked_add(other)
      .expect("overflow when adding duration to instant")
  }
}

impl AddAssign<Duration> for Instant {
  fn add_assign(&mut self, other: Duration) {
    self.0.add_assign(other);
  }
}

impl Sub<Duration> for Instant {
  type Output = Instant;

  fn sub(self, other: Duration) -> Instant {
    self
      .checked_sub(other)
      .expect("overflow when subtracting duration from instant")
  }
}

impl SubAssign<Duration> for Instant {
  fn sub_assign(&mut self, other: Duration) {
    self.0.sub_assign(other);
  }
}

impl Sub<Instant> for Instant {
  type Output = Duration;

  fn sub(self, other: Instant) -> Duration {
    self.duration_since(other)
  }
}

impl ToAbsoluteTime for Instant {
  fn to_absolute_time(&self) -> AbsoluteTime {
    self.0.to_absolute_time()
  }
}

impl fmt::Debug for Instant {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}
