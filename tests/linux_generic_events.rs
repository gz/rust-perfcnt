#![feature(alloc, heap_api)]

extern crate perfcnt;
extern crate alloc;

use perfcnt::{PerfCounter, AbstractPerfCounter};
use perfcnt::linux::{SoftwareEventType, PerfCounterBuilderLinux};
use alloc::heap::{allocate};

#[test]
pub fn test_page_faults() {
    let pc: PerfCounter = PerfCounterBuilderLinux::from_software_event(SoftwareEventType::PageFaults)
        .exclude_idle()
        .exclude_kernel()
        .finish()
        .expect("Could not create counter");

    let p: *mut u8 = unsafe {
        // Make sure the buffer is big enough such that it is going to be
        // mmaped and not coming from some pre-allocated buffers
        allocate(1024*1024*16, 4096)
    };

    pc.start().expect("Can not start the counter");
    // Touch two pages:
    unsafe {
        std::ptr::write(p, 0x1);
        std::ptr::write(p.offset(4096), 0x01);
    }
    pc.stop().expect("Can not start the counter");

    assert_eq!(2, pc.read().expect("Can not read the counter"));

}
