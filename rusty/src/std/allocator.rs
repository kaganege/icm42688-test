use super::ffi;
pub use core::alloc::*;

/// The global allocator type.
#[derive(Default)]
pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    crate::malloc(layout.size() as u32) as *mut u8
  }
  unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
    crate::free(ptr as *mut ffi::c_void);
  }
}

/// The static global allocator.
#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator;
