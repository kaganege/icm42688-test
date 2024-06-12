use super::thread;
use alloc::string::ToString;
use core::panic::PanicInfo;
use core::time::Duration;

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
  cortex_m::interrupt::disable();

  loop {
    eprintln!("{}", panic_info.to_string().as_str());

    thread::sleep(Duration::from_secs(5));
  }
}
