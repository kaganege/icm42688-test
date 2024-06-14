use crate::{
  gpio, spi_deinit, spi_get_baudrate, spi_init, spi_inst, spi_read_blocking, spi_set_baudrate,
  spi_write_blocking, spi_write_read_blocking, PICO_DEFAULT_SPI_CSN_PIN,
};
use cortex_m::asm::nop;

mod defines;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
  UnknownError,
}

pub type Result<T> = core::result::Result<T, Error>;

#[allow(unused)]
#[derive(Debug, Default)]
pub enum Instance {
  #[default]
  SPI0,
  SPI1,
}

/// Serial Peripheral Interface
///
/// # Example
///
/// ```
/// let mut spi = SPI::new(Instance::SPI0);
/// spi.init(48000);
/// ```
///
#[derive(Debug)]
pub struct SPI {
  spi: *mut spi_inst,
  cs: gpio::Pin,
  initialized: bool,
}

impl Default for SPI {
  #[must_use]
  fn default() -> Self {
    Self {
      spi: defines::SPI0_PTR,
      cs: gpio::Pin::new_init(PICO_DEFAULT_SPI_CSN_PIN),
      initialized: Default::default(),
    }
  }
}

#[allow(unused)]
impl SPI {
  #[must_use]
  pub fn new(spi: Instance, cs_pin: gpio::Pin) -> Self {
    use Instance::*;

    let spi = match spi {
      SPI0 => defines::SPI0_PTR,
      SPI1 => defines::SPI1_PTR,
    };

    Self {
      spi,
      cs: cs_pin,
      ..Default::default()
    }
  }

  /// Initialize SPI instance
  ///
  /// Puts the SPI into a known state, and enable it.
  /// Must be called before other functions.
  pub fn init(&mut self, baudrate: u32) -> u32 {
    self.cs.set_direction(gpio::Direction::OUT);
    self.cs.put(true);

    self.initialized = true;
    unsafe { spi_init(self.spi, baudrate) }
  }

  /// Set SPI baudrate
  pub fn set_baudrate(&self, baudrate: u32) -> u32 {
    unsafe { spi_set_baudrate(self.spi, baudrate) }
  }

  /// Get SPI baudrate
  pub fn get_baudrate(&self) -> u32 {
    unsafe { spi_get_baudrate(self.spi) }
  }

  fn cs_select(&self) {
    if self.cs.get() {
      for _ in 0..3 {
        nop();
      }

      self.cs.put(false);

      for _ in 0..3 {
        nop();
      }
    }
  }

  fn cs_deselect(&self) {
    if !self.cs.get() {
      for _ in 0..3 {
        nop();
      }

      self.cs.put(true);

      for _ in 0..3 {
        nop();
      }
    }
  }

  /// Read from an SPI device
  ///
  /// Read `N` bytes from SPI. Blocks until all data is transferred. No timeout,
  /// as SPI hardware always transfers at a known data rate. `repeated_tx_data` is
  /// output repeatedly on TX as data is read in from RX. Generally this can be
  /// 0, but some devices require a specific value here, e.g. SD cards expect
  /// `0xff`
  pub fn read<const N: usize>(&self, repeated_tx_data: u8, stop: bool) -> Result<[u8; N]> {
    let mut dst: [u8; N] = [0x00; N];
    self.cs_select();
    let bytes = unsafe { spi_read_blocking(self.spi, repeated_tx_data, dst.as_mut_ptr(), N) };
    if stop {
      self.cs_deselect();
    }

    if bytes < 0 {
      Err(Error::UnknownError)
    } else {
      Ok(dst)
    }
  }

  /// Write to an SPI device, blocking.
  ///
  /// Write `N` bytes from `src` to SPI, and discard any data received back Blocks
  /// until all data is transferred. No timeout, as SPI hardware always
  /// transfers at a known data rate.
  pub fn write<const N: usize>(&self, src: &[u8; N], stop: bool) -> Result<()> {
    self.cs_select();
    let bytes = unsafe { spi_write_blocking(self.spi, src.as_ptr(), N) };
    if stop {
      self.cs_deselect();
    }

    if bytes < 0 {
      Err(Error::UnknownError)
    } else {
      Ok(())
    }
  }

  /// Write/Read to/from an SPI device.
  ///
  /// Write `N` bytes from `src` to SPI. Simultaneously read `N` bytes from SPI.
  /// Blocks until all data is transferred. No timeout, as SPI hardware always
  /// transfers at a known data rate.
  pub fn write_read<const N: usize>(&self, src: &[u8; N]) -> Result<[u8; N]> {
    let mut dst: [u8; N] = [0x00; N];
    self.cs_select();
    let bytes = unsafe { spi_write_read_blocking(self.spi, src.as_ptr(), dst.as_mut_ptr(), N) };
    self.cs_deselect();

    if bytes < 0 {
      Err(Error::UnknownError)
    } else {
      Ok(dst)
    }
  }
}

impl Drop for SPI {
  /// Deinitialize SPI instance if it initialized
  ///
  /// Puts the SPI into a disabled state. Init will need to be called to
  /// reenable the device functions.
  fn drop(&mut self) {
    if self.initialized {
      unsafe {
        spi_deinit(self.spi);
      }
    }
  }
}
