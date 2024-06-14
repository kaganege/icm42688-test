// #![allow(unused)]

use crate::{i2c, spi};

#[derive(Debug)]
pub enum BusError {
  I2C(i2c::Error),
  SPI(spi::Error),
}

/// Any type of error which may occur while interacting with the device
#[derive(Debug)]
pub enum Error {
  /// The sensor is not ready
  NotReady,
  /// Some error originating from the communication bus
  BusError(BusError),
  /// Some error resulting from interacting with the device
  SensorError(SensorError),
}

/// Any type of error specific to this device
#[derive(Debug)]
pub enum SensorError {
  /// The chip at the specified address is not reporting the correct self
  /// identification code.
  ///
  /// For I²C this is most likely if the ID change jumper is in the wrong
  /// state or there is anther chip on the bus with this address.
  BadChip,
  /// Returned if the register bank is set to a invalid value
  ///
  /// There are 4 banks, 0-3
  BankOutOfRange,
  /// Attempted to write to a read-only register
  WriteToReadOnly,
  /// Attempted to create an AccelRange or GyroRange enum from an invalid
  /// discriminant
  InvalidDiscriminant,
}

impl From<SensorError> for Error {
  fn from(err: SensorError) -> Self {
    Error::SensorError(err)
  }
}
