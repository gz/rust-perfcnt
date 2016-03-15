extern crate libc;
#[macro_use]
extern crate x86;
#[macro_use]
extern crate bitflags;

#[cfg(target_os = "linux")]
extern crate mmap;

pub mod linux;
pub use linux::{PerfCounter};

use std::io;

/// Abstract trait to control performance counters.
pub trait AbstractPerfCounter {
    /// Reset performance counter.
    fn reset(&self) -> Result<(), io::Error>;

    /// Start measuring.
    fn start(&self) -> Result<(), io::Error>;

    /// Stop measuring.
    fn stop(&self) -> Result<(), io::Error>;

    /// Read the counter value.
    fn read(&mut self) -> Result<u64, io::Error>;
}
