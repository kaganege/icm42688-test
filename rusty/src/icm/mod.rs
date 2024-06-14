#![allow(unused)]

use crate::i2c::{self, I2C};
use crate::spi::{self, SPI};
use crate::std::{thread, time::Duration};
pub use accelerometer::{
  error::Error as AccelerometerError,
  vector::{F32x3, I16x3},
  Accelerometer, RawAccelerometer,
};
use config::*;
use core::f32::consts::PI;
use core::fmt;
use core::ptr::addr_of_mut;
use error::*;
use register::*;

pub use config::{
  AccelBandwidth, AccelODR, AccelRange, Address, GyroBandwidth, GyroODR, GyroRange, I2CSlewRate,
  PowerMode,
};
pub use error::Error;

pub mod config;
mod error;
mod register;

pub type Result<T> = core::result::Result<T, Error>;

const GRAVITY: f32 = 9.80665;

pub enum CommunicationProtocol {
  I2C(i2c::Device),
  SPI(SPI),
}

/// ICM-42688 driver
pub struct ICM42688 {
  comm: CommunicationProtocol,
  ready: bool,
}

impl Default for ICM42688 {
  fn default() -> Self {
    Self {
      // i2c: I2C::new(unsafe { addr_of_mut!(crate::i2c0_inst) }),
      // address: Default::default(),
      comm: CommunicationProtocol::I2C(i2c::Device::new(
        I2C::new(unsafe { addr_of_mut!(crate::i2c0_inst) }),
        Address::default() as _,
      )),
      ready: false,
    }
  }
}

impl ICM42688 {
  pub(crate) const DEVICE_ID: u8 = 0x47;

  pub fn new(protocol: CommunicationProtocol) -> Self {
    Self {
      comm: protocol,
      ..Default::default()
    }
  }

  pub fn with_i2c(i2c: I2C, address: Address) -> Self {
    Self::new(CommunicationProtocol::I2C(i2c::Device::new(
      i2c,
      address as _,
    )))
  }

  pub fn with_spi(spi: SPI) -> Self {
    Self::new(CommunicationProtocol::SPI(spi))
  }

  pub fn init(&mut self) -> Result<()> {
    self.ready = true;

    if Self::DEVICE_ID != self.device_id()? {
      return Err(Error::SensorError(SensorError::BadChip));
    }
    debug!("Passed device id control");

    self.soft_reset()?;
    debug!("Soft reset");

    thread::sleep_ms(1);

    // Make sure that any configuration has been restored to the default values when
    // initializing the driver.
    self.set_accel_range(AccelRange::default())?;
    debug!("set_accel_range");
    self.set_gyro_range(GyroRange::default())?;
    debug!("set_gyro_range");

    // The IMU uses `PowerMode::Sleep` by default, which disables both the accel and
    // gyro, so we enable them both during driver initialization.
    self.set_power_mode(PowerMode::SixAxisLowNoise)?;
    debug!("set_power_mode");

    Ok(())
  }

  /// Read the ID of the connected device
  pub fn device_id(&self) -> Result<u8> {
    self.read_register(&Bank0::WHO_AM_I)
  }

  /// soft reset the device
  pub fn soft_reset(&self) -> Result<()> {
    self.update_register(&Bank0::DEVICE_CONFIG, 0x01, 0b0000_0001)
  }

  /// soft reset the device
  pub fn set_i2c_slew_rate(&self, slew_rate: I2CSlewRate) -> Result<()> {
    self.write_register(&Bank0::DRIVE_CONFIG, (slew_rate as u8) << 3)?;
    Ok(())
  }

  /// Return the normalized gyro data for each of the three axes
  pub fn gyro_norm(&self) -> Result<F32x3> {
    let range = self.gyro_range()?;
    let scale = range.scale_factor();

    // Scale the raw Gyroscope data using the appropriate factor based on the
    // configured range.
    let raw = self.gyro_raw()?;
    let x = raw.x as f32 / scale;
    let y = raw.y as f32 / scale;
    let z = raw.z as f32 / scale;

    Ok(F32x3::new(x, y, z))
  }

  /// Read the raw gyro data for each of the three axes
  pub fn gyro_raw(&self) -> Result<I16x3> {
    let x = self.read_register_i16(&Bank0::GYRO_DATA_X1, &Bank0::GYRO_DATA_X0)?;
    let y = self.read_register_i16(&Bank0::GYRO_DATA_Y1, &Bank0::GYRO_DATA_Y0)?;
    let z = self.read_register_i16(&Bank0::GYRO_DATA_Z1, &Bank0::GYRO_DATA_Z0)?;

    Ok(I16x3::new(x, y, z))
  }

  /// Read the built-in temperature sensor and return the value in degrees
  /// centigrade
  pub fn temperature(&self) -> Result<f32> {
    let raw = self.temperature_raw()? as f32;
    let deg = (raw / 132.48) + 25.0;

    Ok(deg)
  }

  /// Read the raw data from the built-in temperature sensor
  pub fn temperature_raw(&self) -> Result<i16> {
    self.read_register_i16(&Bank0::TEMP_DATA1, &Bank0::TEMP_DATA0)
  }

  /// Return the currently configured power mode
  pub fn power_mode(&self) -> Result<PowerMode> {
    //  `GYRO_MODE` occupies bits 3:2 in the register
    // `ACCEL_MODE` occupies bits 1:0 in the register
    let bits = self.read_register(&Bank0::PWR_MGMT0)? & 0x3F;

    Ok(PowerMode::try_from(bits)?)
  }

  /// Set the power mode of the IMU
  pub fn set_power_mode(&self, mode: PowerMode) -> Result<()> {
    self.update_register(&Bank0::PWR_MGMT0, mode.bits(), PowerMode::BITMASK)
  }

  /// Return the currently configured accelerometer range
  pub fn accel_range(&self) -> Result<AccelRange> {
    // `ACCEL_UI_FS_SEL` occupies bits 6:5 in the register
    let fs_sel = self.read_register(&Bank0::ACCEL_CONFIG0)? >> 5;

    Ok(AccelRange::try_from(fs_sel)?)
  }

  /// Set the range of the accelerometer
  pub fn set_accel_range(&self, range: AccelRange) -> Result<()> {
    self.update_register(&Bank0::ACCEL_CONFIG0, range.bits(), AccelRange::BITMASK)
  }

  /// Return the currently configured gyroscope range
  pub fn gyro_range(&self) -> Result<GyroRange> {
    // `GYRO_UI_FS_SEL` occupies bits 6:5 in the register
    let fs_sel = self.read_register(&Bank0::GYRO_CONFIG0)? >> 5;

    Ok(GyroRange::try_from(fs_sel)?)
  }

  /// Set the range of the gyro
  pub fn set_gyro_range(&self, range: GyroRange) -> Result<()> {
    self.update_register(&Bank0::GYRO_CONFIG0, range.bits(), GyroRange::BITMASK)
  }

  /// Return the currently configured output data rate for the gyroscope
  pub fn gyro_odr(&self) -> Result<GyroODR> {
    // `GYRO_ODR` occupies bits 3:0 in the register
    let odr = self.read_register(&Bank0::GYRO_CONFIG0)? & 0xF;

    Ok(GyroODR::try_from(odr)?)
  }

  /// Set the output data rate of the gyroscope
  pub fn set_gyro_odr(&self, odr: GyroODR) -> Result<()> {
    self.update_register(&Bank0::GYRO_CONFIG0, odr.bits(), GyroODR::BITMASK)
  }

  pub fn gyro_bandwith(&self) -> Result<GyroBandwidth> {
    // `GYRO_UI_FILT_BW` occupies bits 2:0 in the register
    let bw_sel = self.read_register(&Bank0::GYRO_ACCEL_CONFIG0)? & 0x0F;

    Ok(GyroBandwidth::try_from(bw_sel)?)
  }

  /// Set the gyro_bandwith filter of the gyro
  pub fn set_gyro_bw(&self, range: GyroBandwidth) -> Result<()> {
    self.update_register(
      &Bank0::GYRO_ACCEL_CONFIG0,
      range.bits(),
      GyroBandwidth::BITMASK,
    )
  }

  /// Return the currently configured output data rate for the accelerometer
  pub fn accel_odr(&self) -> Result<AccelODR> {
    // `ACCEL_ODR` occupies bits 3:0 in the register
    let odr = self.read_register(&Bank0::ACCEL_CONFIG0)? & 0xF;

    Ok(AccelODR::try_from(odr)?)
  }

  /// Set the output data rate of the accelerometer
  pub fn set_accel_odr(&self, odr: AccelODR) -> Result<()> {
    self.update_register(&Bank0::ACCEL_CONFIG0, odr.bits(), AccelODR::BITMASK)
  }

  pub fn accel_bandwith(&self) -> Result<AccelBandwidth> {
    // `ACCEL_UI_FILT_BW` occupies bits 2:0 in the register
    let bw_sel = self.read_register(&Bank0::GYRO_ACCEL_CONFIG0)? >> 4 & 0x0F;
    let bw = AccelBandwidth::try_from(bw_sel)?;

    Ok(bw)
  }

  /// Set the accel_bandwith filter of the accel-meter
  pub fn set_accel_bw(&self, range: AccelBandwidth) -> Result<()> {
    self.update_register(
      &Bank0::GYRO_ACCEL_CONFIG0,
      range.bits(),
      AccelBandwidth::BITMASK,
    )
  }

  /// read time stampe from register
  pub fn read_tmst(&self) -> Result<u16> {
    let ped_cnt = self.read_register_u16(&Bank0::TMST_FSYNCH, &Bank0::TMST_FSYNCL)?;
    Ok(ped_cnt)
  }

  /// read current fifo buffer level, available to read
  pub fn read_fifo_cnt(&self) -> Result<u16> {
    let fifo_cnt = self.read_register_u16(&Bank0::FIFO_COUNTH, &Bank0::FIFO_COUNTL)?;
    Ok(fifo_cnt)
  }

  pub fn read_fifo(&self, addr: u8) -> Result<u8> {
    if !self.ready {
      Err(Error::NotReady)
    } else {
      use CommunicationProtocol::*;

      //let mut buffer = [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];
      let buffer: [u8; 8] = match self.comm {
        I2C(ref device) => device
          .write_read(&[addr])
          .map_err(|e| Error::BusError(BusError::I2C(e)))?,
        SPI(ref device) => {
          device
            .write(&[addr], false)
            .map_err(|e| Error::BusError(BusError::SPI(e)))?;
          thread::sleep_ms(10);
          device
            .read(0, true)
            .map_err(|e| Error::BusError(BusError::SPI(e)))?
        }
      };

      Ok(buffer[0])
    }
  }

  fn read_bank(&self, bank: RegisterBank, reg: &dyn Register) -> Result<u8> {
    // See "ACCESSING MREG1, MREG2 AND MREG3 REGISTERS" (page 40)

    // Wait until the internal clock is running prior to writing.
    // while self.read_register(&Bank0::MCLK_RDY)? != 0x9 {}

    // Set Bank to read from
    self.write_register(&Bank0::REG_BANK_SEL, bank.blk_sel())?;
    // self.write_reg(&Bank0::MADDR_R, reg.addr())?;
    thread::sleep(Duration::from_millis(10));

    // Read a value from the register.
    let result = self.read_register(reg)?;
    thread::sleep(Duration::from_millis(10));

    // Reset block selection registers.
    self.update_register(&Bank0::REG_BANK_SEL, 0x00, 0x07)?;

    Ok(result)
  }

  fn write_bank(&self, bank: RegisterBank, reg: &dyn Register, value: u8) -> Result<()> {
    // Set Bank to write to
    self.update_register(&Bank0::REG_BANK_SEL, bank.blk_sel(), 0x07)?;

    // Write the value to the register.
    self.write_register(reg, value)?;
    thread::sleep(Duration::from_millis(10));

    // Reset block selection registers.
    self.update_register(&Bank0::REG_BANK_SEL, 0x00, 0x07)?;

    Ok(())
  }

  fn set_bank(&self, bank: RegisterBank) -> Result<()> {
    // Set Bank to write to
    self.update_register(&Bank0::REG_BANK_SEL, bank.blk_sel(), 0x07)?;
    Ok(())
  }

  fn read_register(&self, reg: &dyn Register) -> Result<u8> {
    if !self.ready {
      Err(Error::NotReady)
    } else {
      use CommunicationProtocol::*;

      let data: [u8; 1] = match self.comm {
        I2C(ref device) => device
          .write_read(&[reg.addr()])
          .map_err(|e| Error::BusError(BusError::I2C(e)))?,
        SPI(ref device) => device
          .write_read(&[reg.addr()])
          .map_err(|e| Error::BusError(BusError::SPI(e)))?,
      };

      Ok(data[0])
    }
  }

  /// Read two registers and combine them into a single value.
  fn read_register_i16(&self, reg_hi: &dyn Register, reg_lo: &dyn Register) -> Result<i16> {
    let data_lo = self.read_register(reg_lo)?;
    let data_hi = self.read_register(reg_hi)?;

    let data = i16::from_be_bytes([data_hi, data_lo]);

    Ok(data)
  }

  /// Read two registers and combine them into a single value.
  fn read_register_u16(&self, reg_hi: &dyn Register, reg_lo: &dyn Register) -> Result<u16> {
    let data_lo = self.read_register(reg_lo)?;
    let data_hi = self.read_register(reg_hi)?;

    let data = u16::from_be_bytes([data_hi, data_lo]);

    Ok(data)
  }

  fn write_register(&self, reg: &dyn Register, value: u8) -> Result<()> {
    if reg.read_only() {
      Err(Error::SensorError(SensorError::WriteToReadOnly))
    } else if !self.ready {
      Err(Error::NotReady)
    } else {
      use CommunicationProtocol::*;

      match self.comm {
        I2C(ref device) => device
          .write(&[reg.addr()], true)
          .map_err(|e| Error::BusError(BusError::I2C(e))),
        SPI(ref device) => device
          .write(&[reg.addr()], true)
          .map_err(|e| Error::BusError(BusError::SPI(e))),
      }
    }
  }

  fn update_register(&self, reg: &dyn Register, value: u8, mask: u8) -> Result<()> {
    if reg.read_only() {
      Err(Error::SensorError(SensorError::WriteToReadOnly))
    } else {
      let current = self.read_register(reg)?;
      let value = (current & !mask) | (value & mask);

      self.write_register(reg, value)
    }
  }
}

impl RawAccelerometer<I16x3> for ICM42688 {
  type Error = Error;

  fn accel_raw(&mut self) -> core::result::Result<I16x3, AccelerometerError<Self::Error>> {
    let x = self.read_register_i16(&Bank0::ACCEL_DATA_X1, &Bank0::ACCEL_DATA_X0)?;
    let y = self.read_register_i16(&Bank0::ACCEL_DATA_Y1, &Bank0::ACCEL_DATA_Y0)?;
    let z = self.read_register_i16(&Bank0::ACCEL_DATA_Z1, &Bank0::ACCEL_DATA_Z0)?;

    Ok(I16x3::new(x, y, z))
  }
}

impl Accelerometer for ICM42688 {
  type Error = Error;

  fn accel_norm(&mut self) -> core::result::Result<F32x3, AccelerometerError<Self::Error>> {
    let range = self.accel_range()?;
    let scale = range.scale_factor();

    // Scale the raw Accelerometer data using the appropriate factor based on the
    // configured range.
    let raw = self.accel_raw()?;
    let x = raw.x as f32 / scale;
    let y = raw.y as f32 / scale;
    let z = raw.z as f32 / scale;

    Ok(F32x3::new(x, y, z))
  }

  fn sample_rate(&mut self) -> core::result::Result<f32, AccelerometerError<Self::Error>> {
    let odr = self.accel_odr()?;

    Ok(odr.as_f32())
  }
}
