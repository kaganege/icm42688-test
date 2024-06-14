use alloc::vec::Vec;

use crate::error;
use crate::std::time::{Duration, Instant, ToAbsoluteTime};
use crate::{
  i2c_deinit, i2c_init, i2c_inst, i2c_read_blocking, i2c_read_blocking_until, i2c_write_blocking,
  i2c_write_blocking_until,
};

#[allow(unused)]
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
  AddressNotAcknowledged,
  NoDevicePresent,
  Both,
  Timeout,
}

pub type Result<T> = core::result::Result<T, Error>;

pub struct I2C(*mut i2c_inst);

#[allow(unused)]
impl I2C {
  pub fn new(i2c: *mut i2c_inst) -> Self {
    Self(i2c)
  }

  pub fn init(&self, baudrate: u32) -> u32 {
    unsafe { i2c_init(self.0, baudrate) }
  }

  pub fn read_timeout<const N: usize>(
    &self,
    address: u8,
    timeout: Duration,
    stop: bool,
  ) -> Result<[u8; N]> {
    let mut dst: [u8; N] = [0x00; N];
    let bytes = unsafe {
      i2c_read_blocking_until(
        self.0,
        address,
        dst.as_mut_ptr(),
        N,
        !stop,
        (Instant::now() + timeout).to_absolute_time(),
      )
    };

    const PICO_ERROR_GENERIC: i32 = error::PicoErrorCodes::GENERIC as _;
    const PICO_ERROR_TIMEOUT: i32 = error::PicoErrorCodes::TIMEOUT as _;

    match bytes {
      PICO_ERROR_GENERIC => Err(Error::Both),
      PICO_ERROR_TIMEOUT => Err(Error::Timeout),
      _ => Ok(dst),
    }
  }

  pub fn read<const N: usize>(&self, address: u8, stop: bool) -> Result<[u8; N]> {
    let mut dst: [u8; N] = [0x00; N];
    let bytes = unsafe { i2c_read_blocking(self.0, address, dst.as_mut_ptr(), N, !stop) };

    if bytes == error::PicoErrorCodes::GENERIC as _ {
      return Err(Error::Both);
    }

    Ok(dst)
  }

  pub fn write_timeout<const N: usize>(
    &self,
    address: u8,
    src: &[u8; N],
    timeout: Duration,
    stop: bool,
  ) -> Result<()> {
    let bytes = unsafe {
      i2c_write_blocking_until(
        self.0,
        address,
        src.as_ptr(),
        N,
        !stop,
        (Instant::now() + timeout).to_absolute_time(),
      )
    };

    const PICO_ERROR_GENERIC: i32 = error::PicoErrorCodes::GENERIC as _;
    const PICO_ERROR_TIMEOUT: i32 = error::PicoErrorCodes::TIMEOUT as _;

    match bytes {
      PICO_ERROR_GENERIC => Err(Error::Both),
      PICO_ERROR_TIMEOUT => Err(Error::Timeout),
      _ => Ok(()),
    }
  }

  pub fn write<const N: usize>(&self, address: u8, src: &[u8; N], stop: bool) -> Result<()> {
    let bytes = unsafe { i2c_write_blocking(self.0, address, src.as_ptr(), N, !stop) };

    if bytes == error::PicoErrorCodes::GENERIC as _ {
      Err(Error::Both)
    } else {
      Ok(())
    }
  }

  pub fn write_read_timeout<const N: usize, const M: usize>(
    &self,
    address: u8,
    src: &[u8; N],
    timeout: Duration,
  ) -> Result<[u8; M]> {
    self.write_timeout(address, src, timeout, false)?;
    self.read_timeout(address, timeout, true)
  }

  pub fn write_read<const N: usize, const M: usize>(
    &self,
    address: u8,
    src: &[u8; N],
  ) -> Result<[u8; M]> {
    self.write(address, src, false)?;
    self.read(address, true)
  }

  pub fn scan(&self) -> Vec<u8> {
    debugln!(min "   0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F");

    (0..(1 << 7)).fold(Vec::new(), |mut list, addr| {
      if addr % 16 == 0 {
        debug!(min "{} ", if addr == 0 { 0 } else { addr / 16 });
      }

      if !reserved_addr(addr) && self.read::<1>(addr, true).is_ok() {
        list.push(addr);
        debug!(min "@");
      }

      if addr % 16 == 15 {
        debugln!(min);
      } else {
        debug!(min "  ");
      }

      list
    })
  }
}

impl Drop for I2C {
  fn drop(&mut self) {
    unsafe {
      i2c_deinit(self.0);
    }
  }
}

fn reserved_addr(addr: u8) -> bool {
  return (addr & 0x78) == 0 || (addr & 0x78) == 0x78;
}

pub struct Device {
  i2c: I2C,
  address: u8,
}

#[allow(unused)]
impl Device {
  pub fn new(i2c: I2C, address: u8) -> Self {
    Self { i2c, address }
  }

  pub fn read_timeout<const N: usize>(&self, timeout: Duration, stop: bool) -> Result<[u8; N]> {
    self.i2c.read_timeout(self.address, timeout, stop)
  }

  pub fn read<const N: usize>(&self, stop: bool) -> Result<[u8; N]> {
    self.i2c.read(self.address, stop)
  }

  pub fn write_timeout<const N: usize>(
    &self,
    src: &[u8; N],
    timeout: Duration,
    stop: bool,
  ) -> Result<()> {
    self.i2c.write_timeout(self.address, src, timeout, stop)
  }

  pub fn write<const N: usize>(&self, src: &[u8; N], stop: bool) -> Result<()> {
    self.i2c.write(self.address, src, stop)
  }

  pub fn write_read_timeout<const N: usize, const M: usize>(
    &self,
    src: &[u8; N],
    timeout: Duration,
  ) -> Result<[u8; M]> {
    self.i2c.write_read_timeout(self.address, src, timeout)
  }

  pub fn write_read<const N: usize, const M: usize>(&self, src: &[u8; N]) -> Result<[u8; M]> {
    self.i2c.write_read(self.address, src)
  }
}
