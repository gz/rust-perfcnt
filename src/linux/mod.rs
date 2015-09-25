use std::fs::File;
use std::os::unix::io::FromRawFd;
use std::io::Read;
use std::io;
use std::mem;
use std::slice::from_raw_parts_mut;


#[allow(dead_code, non_camel_case_types)]
mod hw_breakpoint;
#[allow(dead_code, non_camel_case_types)]
pub mod perf_event;


pub const IOCTL: usize = 16;
pub const PERF_EVENT_OPEN: usize = 298;

pub fn perf_event_open(hw_event: perf_event::perf_event_attr,
                       pid: perf_event::__kernel_pid_t,
                       cpu:  ::libc::c_int,
                       group_fd:  ::libc::c_int,
                       flags:  ::libc::c_int) -> isize {
    unsafe {
        syscall!(PERF_EVENT_OPEN, &hw_event as *const perf_event::perf_event_attr as usize, pid, cpu, group_fd, flags) as isize
    }
}

pub fn ioctl(fd: ::libc::c_int, request: u64, value: ::libc::c_int) -> isize {
    unsafe {
        syscall!(IOCTL, fd, request, value) as isize
    }
}

pub fn read_counter(fd: ::libc::c_int) -> Result<u64, io::Error> {
    unsafe {
        let mut f = File::from_raw_fd(fd);
        let mut value: [u8; 8] = Default::default();
        let result = try!(f.read(&mut value));

        Ok(mem::transmute::<[u8; 8], u64>(value))
    }
}