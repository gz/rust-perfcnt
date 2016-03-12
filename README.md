# perfcnt [![Build Status](https://travis-ci.org/gz/rust-perfcnt.svg)](https://travis-ci.org/gz/rust-perfcnt) [![Crates.io](https://img.shields.io/crates/v/perfcnt.svg)](https://crates.io/crates/perfcnt)

A library to program performance counters in rust.

## Example library usage
```rust
let mut pc: PerfCounter = PerfCounterBuilderLinux::from_hardware_event(HardwareEventType::CacheMisses)
pc.start().expect("Can not start the counter");
pc.stop().expect("Can not start the counter");
let res = pc.read().expect("Can not read the counter");
println!("Measured {} cache misses.", res);
```
  * See examples/ directory for more code-snippets on how-to use the library to create counters.

## Documentation
  * [API Documentation](http://gz.github.io/rust-perfcnt/perfcnt/)

## Provided Programs
  * *perfcnt-list*: Lists all architecture specific events available on the current machine (currently only supports Intel x86).

## Known limitations
 * Linux support without breakpoints and tracepoints
 * No Windows or MacOS X support
 * Missing raw AMD and ARM aarch64 events
