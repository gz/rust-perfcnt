#![feature(alloc, heap_api)]

extern crate perfcnt;
extern crate alloc;

use perfcnt::{PerfCounter, AbstractPerfCounter};
use perfcnt::linux::{SoftwareEventType, PerfCounterBuilderLinux, ReadFormat};
use alloc::heap::{allocate};

#[test]
pub fn test_page_faults() {
    let mut pc: PerfCounter = PerfCounterBuilderLinux::from_software_event(SoftwareEventType::PageFaults)
        .exclude_idle()
        .exclude_kernel()
        .add_read_format(ReadFormat::TotalTimeEnabled)
        .add_read_format(ReadFormat::TotalTimeRunning)
        .add_read_format(ReadFormat::FormatId)
        .finish()
        .expect("Could not create counter");

    //.add_read_format(ReadFormat::FormatGroup)

    let size = 1024*1024*16;
    let page_size = 4096;
    let p: *mut u8 = unsafe {
        // Make sure the buffer is big enough such that it is going to be
        // mmaped and not coming from some pre-allocated buffers
        allocate(size, page_size)
    };

    pc.reset().expect("Can not reset");
    pc.start().expect("Can not start the counter");
    // Touch two pages:
    unsafe {
        std::ptr::write(p, 0x1);
        std::ptr::write(p.offset(((size/page_size/2)*page_size) as isize), 0x01);
    }
    pc.stop().expect("Can not start the counter");

    // Should be ~= 2
    let res = pc.read_fd().expect("Can not read the counter");
    assert_eq!(res.value, 2);
    println!("{:?}", res);
    //assert!(res.value <= 4);
}
