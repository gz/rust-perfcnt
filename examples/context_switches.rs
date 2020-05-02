extern crate perfcnt;
extern crate x86;

use perfcnt::linux::PerfCounterBuilderLinux as Builder;
use perfcnt::linux::SoftwareEventType as Software;
use perfcnt::{AbstractPerfCounter, PerfCounter};

pub fn main() {
    let mut pc: PerfCounter = Builder::from_software_event(Software::ContextSwitches)
        .on_cpu(0)
        .for_all_pids()
        .finish()
        .expect("Could not create counter");

    pc.start().expect("Can not start the counter");
    std::thread::sleep(std::time::Duration::new(1, 0));
    pc.stop().expect("Can not stop the counter");

    println!(
        "Context Switches/s: {:?}",
        pc.read().expect("Can not read counter")
    );
    pc.reset().expect("Can not reset the counter");
}
