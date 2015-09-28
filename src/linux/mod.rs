use std::fs::File;
use std::os::unix::io::FromRawFd;
use std::io::Read;
use std::io;
use std::mem;

#[allow(dead_code, non_camel_case_types)]
mod hw_breakpoint;
#[allow(dead_code, non_camel_case_types)]
mod perf_event;

use ::PerfCounterControl;

const IOCTL: usize = 16;
const PERF_EVENT_OPEN: usize = 298;

fn perf_event_open(hw_event: perf_event::perf_event_attr,
                       pid: perf_event::__kernel_pid_t,
                       cpu:  ::libc::c_int,
                       group_fd:  ::libc::c_int,
                       flags:  ::libc::c_int) -> isize {
    unsafe {
        syscall!(PERF_EVENT_OPEN, &hw_event as *const perf_event::perf_event_attr as usize, pid, cpu, group_fd, flags) as isize
    }
}

fn ioctl(fd: ::libc::c_int, request: u64, value: ::libc::c_int) -> isize {
    unsafe {
        syscall!(IOCTL, fd, request, value) as isize
    }
}

fn read_counter(fd: ::libc::c_int) -> Result<u64, io::Error> {
    unsafe {
        let mut f = File::from_raw_fd(fd);
        let mut value: [u8; 8] = Default::default();
        try!(f.read(&mut value));
        Ok(mem::transmute::<[u8; 8], u64>(value))
    }
}

pub struct PerfCounter {
    fd: ::libc::c_int
}

impl PerfCounterControl for PerfCounter {

    fn new() -> PerfCounter {
        let mut hw_event: perf_event::perf_event_attr = Default::default();
        hw_event._type = perf_event::PERF_TYPE_RAW;
        hw_event.size = mem::size_of::<perf_event::perf_event_attr>() as u32;
        hw_event.config = perf_event::PERF_COUNT_HW_INSTRUCTIONS as u64;
        hw_event.settings =
            perf_event::EVENT_ATTR_DISABLED |
            perf_event::EVENT_ATTR_EXCLUDE_KERNEL |
            perf_event::EVENT_ATTR_EXCLUDE_HV;

        let pid = 0;
        let cpu = -1;
        let group_fd = -1;
        let flags = 0;

        let fd = perf_event_open(hw_event, pid, cpu, group_fd, flags) as ::libc::c_int;
        if fd < 0 {
            println!("Error opening leader {:?}", fd);
        }

        PerfCounter { fd: fd }
    }

    fn reset(&self) {
        let ret = ioctl(self.fd, perf_event::PERF_EVENT_IOC_RESET, 0);
        assert!(ret == 0);
    }

    fn start(&self) {
        let ret = ioctl(self.fd, perf_event::PERF_EVENT_IOC_ENABLE, 0);
        assert!(ret == 0);
    }

    fn stop(&self) {
        let ret = ioctl(self.fd, perf_event::PERF_EVENT_IOC_DISABLE, 0);
        assert!(ret == 0);
    }

    fn read(&self) -> u64 {
        let c = read_counter(self.fd);
        match c {
            Ok(cnt) => cnt,
            _ => 0,
        }
    }
}
