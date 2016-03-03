extern crate perfcnt;
extern crate x86;

use perfcnt::{PerfCounter, PerfCounterBuilderLinux, AbstractPerfCounter};

pub fn main() {
    let counter = x86::perfcnt::core_counters().unwrap().get("BR_INST_RETIRED.ALL_BRANCHES").unwrap();
    let mut pb = PerfCounterBuilderLinux::new();
    pb.exclude_idle();
    pb.exclude_kernel();
    pb.from_raw_intel_hw_counter(counter);
    let pc: PerfCounter = pb.finish().expect("Could not create counter");

    pc.start().expect("Can not start the counter");
    println!("");
    pc.stop().expect("Can not stop the counter");;

    println!("{}: {:?}", counter.brief_description, pc.read().expect("Can not read counter"));
    pc.reset().expect("Can not reset the counter");
}
