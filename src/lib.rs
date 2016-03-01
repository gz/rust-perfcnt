#![feature(negate_unsigned)]
extern crate libc;
#[macro_use]
extern crate x86;
#[macro_use]
extern crate bitflags;

use x86::perfcnt::intel::description::{IntelPerformanceCounterDescription};

#[cfg(target_os="linux")] #[path="linux/mod.rs"]
mod arch;

pub use arch::{PerfCounter};


/// Abstract trait to control performance counters.
trait PerfCounterControl {
    fn reset(&self);
    fn start(&self);
    fn stop(&self);
    fn read(&self) -> u64;
}


#[test]
fn list_mine() {
    for counter in x86::perfcnt::core_counters() {
        println!("{:?}", counter);
    }
}

#[test]
fn list_them() {
    for counter in x86::perfcnt::core_counters() {
        println!("{:?}", counter);
    }
}

#[test]
fn basic_perfcnt() {
    let pc = arch::PerfCounter::new();

    pc.reset();
    pc.start();
    println!("test");
    pc.stop();

    println!("{:?}", pc.read());
}
