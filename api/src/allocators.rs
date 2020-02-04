extern crate alloc;

use crate::logger;


// - ffi imports --------------------------------------------------------------

extern "C" {
    fn malloc(size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
}


// - global constants ---------------------------------------------------------

const TAG: &str = "wrap::ledc";


// - global allocator ---------------------------------------------------------

use core::alloc::{GlobalAlloc, Layout};

struct LibcAllocator;

unsafe impl GlobalAlloc for LibcAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() > 8 {
            panic!("Unsupported alignment")
        }
        malloc(layout.size()) as *mut u8
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr)
    }
}


#[global_allocator]
static A: LibcAllocator = LibcAllocator;


#[alloc_error_handler]
fn on_oom(layout: Layout) -> ! {
    log!(TAG, "out of memory error: {:?}", layout);
    loop {}
}
