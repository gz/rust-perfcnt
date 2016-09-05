extern crate perfcnt;

use std::io::prelude::*;
use std::fs::File;
use std::env;
use std::path::Path;

use perfcnt::parser::{PerfFile};

fn main() {
    for argument in env::args().skip(1) {
        println!("Parsed perf file: {}", argument);
        println!("----------------------------------------------------------");

        let mut file = File::open(argument).expect("File does not exist");
        let mut buf: Vec<u8> = Vec::with_capacity(2*4096*4096);
        match file.read_to_end(&mut buf) {
            Ok(len) => {
                println!("File read: {:?} bytes", len);
                let pf = PerfFile::new(buf);
                println!("{:?}", pf.header);
                println!("{:?}", pf.sections());
                println!("{:?}", pf.attrs);
                println!("{:?}", pf.data());
            }
            Err(e) => {
                panic!("Can't read {:?}: {}", file, e);
            }
        }
    }
}
