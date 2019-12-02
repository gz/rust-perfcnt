extern crate perfcnt;
extern crate x86;

use perfcnt::linux::PerfCounterBuilderLinux;
use perfcnt::{AbstractPerfCounter, PerfCounter};

pub fn main() {
    let counter_description = x86::perfcnt::intel::events()
        .unwrap()
        .get("BR_INST_RETIRED.ALL_BRANCHES")
        .unwrap();
    let mut pc: PerfCounter =
        PerfCounterBuilderLinux::from_intel_event_description(counter_description)
            .exclude_idle()
            .exclude_kernel()
            .finish()
            .expect("Could not create counter");

    pc.start().expect("Can not start the counter");
    println!("");
    pc.stop().expect("Can not stop the counter");

    println!(
        "{}: {:?}",
        counter_description.brief_description,
        pc.read().expect("Can not read counter")
    );
    pc.reset().expect("Can not reset the counter");
}
