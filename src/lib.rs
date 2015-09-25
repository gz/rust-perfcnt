#![feature(negate_unsigned)]

extern crate libc;
#[macro_use]
extern crate x86;
#[macro_use]
extern crate bitflags;


mod linux;
use std::mem;

#[test]
fn basic_perfcnt_linux() {

    let mut hw_event: linux::perf_event::perf_event_attr = Default::default();
    hw_event._type = linux::perf_event::PERF_TYPE_HARDWARE;
    hw_event.size = mem::size_of::<linux::perf_event::perf_event_attr>() as u32;
    hw_event.config = linux::perf_event::PERF_COUNT_HW_INSTRUCTIONS as u64;
    hw_event.settings = linux::perf_event::EVENT_ATTR_DISABLED | linux::perf_event::EVENT_ATTR_EXCLUDE_KERNEL | linux::perf_event::EVENT_ATTR_EXCLUDE_HV;

    let pid = 0;
    let cpu = -1;
    let group_fd = -1;
    let flags = 0;

    let fd = linux::perf_event_open(hw_event, pid, cpu, group_fd, flags) as ::libc::c_int;
    if fd < 0 {
        println!("Error opening leader {:?}", fd);
    }

    let i1 = linux::ioctl(fd, linux::perf_event::PERF_EVENT_IOC_RESET, 0);
    let i2 = linux::ioctl(fd, linux::perf_event::PERF_EVENT_IOC_ENABLE, 0);
    println!("{:?}", fd);
    let i3 = linux::ioctl(fd, linux::perf_event::PERF_EVENT_IOC_DISABLE, 0);
    println!("{:?} {:?} {:?}", i1, i2, i3);

    let c = linux::read_counter(fd);
    println!("Counter is {:?}", c.unwrap());


}
