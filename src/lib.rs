#![feature(negate_unsigned)]
extern crate libc;
#[macro_use]
extern crate x86;
#[macro_use]
extern crate bitflags;

mod linux;

pub use linux::{PerfCounterBuilder, PerfCounter};


/// Abstract trait to control performance counters.
trait PerfCounterTrait {
    fn reset(&self);
    fn start(&self);
    fn stop(&self);
    fn read(&self) -> u64;
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
    let mut pb = PerfCounterBuilder::new();
    pb.from_raw_intel_hw_counter(counter);
    let pc: PerfCounter = pb.finish();

    pc.reset();
    pc.start();
    println!("test");
    pc.stop();

    println!("{:?}", pc.read());
    println!("{:?}", pc.read());
}
