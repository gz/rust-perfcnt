use phf;
use super::IntelPerformanceCounterDescription;
use super::Counter;
use super::PebsType;
use super::EventCode;
use super::MSRIndex;

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));