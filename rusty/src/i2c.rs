#![allow(unused)]

use crate::error;
use crate::{i2c_deinit, i2c_init, i2c_inst, i2c_read_blocking, i2c_write_blocking};
use core::ptr::addr_of_mut;

#[derive(Debug)]
pub enum Error {
  AddressNotAcknowledged,
  NoDevicePresent,
  Both,
}

pub type Result<T> = core::result::Result<T, Error>;

pub struct I2C(*mut i2c_inst);

impl I2C {
  pub fn new(mut i2c: i2c_inst) -> Self {
    Self(addr_of_mut!(i2c))
  }

  pub fn init(&self, baudrate: u32) -> u32 {
    unsafe { i2c_init(self.0, baudrate) }
  }

  pub fn read<const N: usize>(&self, address: u8, stop: bool) -> Result<[u8; N]> {
    let mut dst: [u8; N] = [0x00; N];
    let bytes = unsafe { i2c_read_blocking(self.0, address, &mut dst as _, N, !stop) };

    if bytes == error::PicoErrorCodes::GENERIC as _ {
      return Err(Error::Both);
    }

    Ok(dst)
  }

  pub fn write<const N: usize>(&self, address: u8, src: &[u8; N], stop: bool) -> Result<()> {
    let bytes = unsafe { i2c_write_blocking(self.0, address, src as _, N, !stop) };

    if bytes == error::PicoErrorCodes::GENERIC as _ || bytes < 0 {
      Err(Error::Both)
    } else {
      Ok(())
    }
  }

  pub fn write_read<const N: usize, const M: usize>(
    &self,
    address: u8,
    src: &[u8; N],
  ) -> Result<[u8; M]> {
    self.write(address, src, false)?;
    self.read(address, true)
  }
}

impl Drop for I2C {
  fn drop(&mut self) {
    unsafe {
      i2c_deinit(self.0);
    }
  }
}
