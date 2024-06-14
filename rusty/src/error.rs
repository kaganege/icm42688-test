/// Common return codes from pico_sdk methods that return a status
#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PicoErrorCodes {
  NONE = 0,
  TIMEOUT = -1,
  GENERIC = -2,
  NO_DATA = -3,
  NOT_PERMITTED = -4,
  INVALID_ARG = -5,
  IO = -6,
  BADAUTH = -7,
  CONNECT_FAILED = -8,
  INSUFFICIENT_RESOURCES = -9,
}
