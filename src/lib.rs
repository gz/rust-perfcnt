//! Example usage:
//!
//! ```no_run
//! use perfcnt::{AbstractPerfCounter, PerfCounter};
//! use perfcnt::linux::{PerfCounterBuilderLinux, HardwareEventType};
//!
//! let mut pc: PerfCounter =
//!     PerfCounterBuilderLinux::from_hardware_event(HardwareEventType::CacheMisses)
//!         .finish().expect("Could not create the counter");
//! pc.start().expect("Can not start the counter");
//! pc.stop().expect("Can not start the counter");
//! let res = pc.read().expect("Can not read the counter");
//! println!("Measured {} cache misses.", res);
//! ```

pub mod linux;
pub use crate::linux::PerfCounter;

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
