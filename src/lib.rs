#![feature(negate_unsigned)]
extern crate libc;
#[macro_use]
extern crate x86;
#[macro_use]
extern crate bitflags;

mod linux;

pub use linux::{PerfCounterBuilderLinux, PerfCounter};
use std::io;

/// Abstract trait to control performance counters.
pub trait AbstractPerfCounter {
    fn reset(&self) -> Result<(), io::Error>;
    fn start(&self) -> Result<(), io::Error>;
    fn stop(&self) -> Result<(), io::Error>;
    fn read(&self) -> Result<u64, io::Error>;
}
