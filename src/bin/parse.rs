extern crate perfcnt;

use std::io::prelude::*;
use std::fs::File;
use std::env;

use perfcnt::parser;

fn main() {
    for argument in env::args().skip(1) {
        println!("Parsed perf file: {}", argument);
        println!("----------------------------------------------------------");

        let mut file = File::open(argument).expect("File does not exist");
        let mut buf: Vec<u8> = Vec::with_capacity(2*4096*4096);
        match file.read_to_end(&mut buf) {
            Ok(len) => {
                println!("File read: {:?} bytes", len);
                let r = perfcnt::parser::parse_perf_data(buf.as_slice());
                println!("{:?}", r);
            }
            Err(e) => {
                panic!("Can't read {:?}: {}", file, e);
            }
        }
    }
}
