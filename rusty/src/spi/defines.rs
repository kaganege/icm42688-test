use crate::spi_inst;

pub const SPI0_BASE: u32 = 0x4003c000;
pub const SPI1_BASE: u32 = 0x40040000;

pub const SPI0_PTR: *mut spi_inst = SPI0_BASE as _;
pub const SPI1_PTR: *mut spi_inst = SPI1_BASE as _;
