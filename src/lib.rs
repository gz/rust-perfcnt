//! Example usage:
//!
//! ```
//! use perfcnt::{AbstractPerfCounter, PerfCounter};
//! use perfcnt::linux::{PerfCounterBuilderLinux, HardwareEventType};
//!
//! let mut pc: PerfCounter =
//!     PerfCounterBuilderLinux::from_hardware_event(HardwareEventType::CacheMisses)
//!         .finish().expect("Can not create the counter");
//! pc.start().expect("Can not start the counter");
//! pc.stop().expect("Can not start the counter");
//! let res = pc.read().expect("Can not read the counter");
//! println!("Measured {} cache misses.", res);
//! ```

extern crate libc;
#[macro_use]
extern crate x86;
#[macro_use]
extern crate bitflags;
extern crate nom;

#[cfg(target_os = "linux")]
extern crate mmap;

pub mod linux;
pub use linux::PerfCounter;

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
