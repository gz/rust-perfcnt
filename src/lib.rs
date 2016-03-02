#![feature(negate_unsigned)]
extern crate libc;
#[macro_use]
extern crate x86;
#[macro_use]
extern crate bitflags;

mod linux;

pub use linux::{PerfCounterBuilder, PerfCounter};
use std::io;


/// Abstract trait to control performance counters.
trait AbstractPerfCounter {
    fn reset(&self) -> Result<(), io::Error>;
    fn start(&self) -> Result<(), io::Error>;
    fn stop(&self) -> Result<(), io::Error>;
    fn read(&self) -> Result<u64, io::Error>;
}


#[test]
fn list_mine() {
    for counter in x86::perfcnt::core_counters() {
        //println!("{:?}", counter);
    }
}

#[test]
fn list_them() {
    for counter in x86::perfcnt::core_counters() {
        //println!("{:?}", counter);
    }
}

#[test]
fn basic_perfcnt() {
    let counter = x86::perfcnt::core_counters().unwrap().get("BR_INST_RETIRED.ALL_BRANCHES").unwrap();
    println!("{:?}", counter.brief_description);
    let mut pb = PerfCounterBuilder::new();
    pb.from_raw_intel_hw_counter(counter);
    let pc: PerfCounter = pb.finish();

    pc.reset().expect("Can not reset the counter");
    pc.start().expect("Can not start the counter");
    println!("");
    pc.stop().expect("Can not stop the counter");;

    println!("{:?}", pc.read().expect("Can not read counter"));
}
