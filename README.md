# perfcnt [![Build Status](https://travis-ci.org/gz/rust-perfcnt.svg)](https://travis-ci.org/gz/rust-perfcnt) [![Crates.io](https://img.shields.io/crates/v/perfcnt.svg)](https://crates.io/crates/perfcnt)

A library to program performance counters in rust.

## Documentation

  * [API Documentation](https://docs.rs/perfcnt/)
  * See the [`examples/`](https://github.com/gz/rust-perfcnt/tree/master/examples) directory for more code-snippets on how to use the library to create counters.

## Provided Programs
  * *perfcnt-list*: Lists all architecture specific events available on the current machine (currently only supports Intel x86).

## Known limitations
 * Linux support without breakpoints and tracepoints
 * No Windows or MacOS X support
 * Missing raw AMD and ARM aarch64 events
