extern crate phf_codegen;
extern crate serde_json;

use std::env;
use std::fs::File;
use std::io::{BufWriter, BufReader, Write};
use std::path::Path;

use serde_json::Value;

fn parse_performance_counters() {
    let f = File::open("Haswell_core_V20.json").unwrap();
    let reader = BufReader::new(f);
    let data: Value = serde_json::from_reader(reader).unwrap();

    if data.is_array() {
        let entries = data.as_array().unwrap();
        for entry in entries.iter() {
            if !entry.is_object() {
                panic!("Expected JSON object.");
            }

            let pcn = entry.as_object().unwrap();

            for (key, value) in pcn.iter() {
                println!("{}={}", key, match *value {
                   Value::U64(v) => format!("{} (u64)", v),
                   Value::String(ref v) => format!("{} (string)", v),
                   _ => format!("other")
               });
            }
        }
    }
    else {
        panic!("JSON data is not an array.");
    }

    //panic!("done");
}

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    write!(&mut file, "static PERFORMANCE_COUNTER_HASWELL: phf::Map<&'static str, IntelPerformanceCounterDescription> = ").unwrap();

    let mut builder = phf_codegen::Map::new();
    let entries = [("hello", "1"), ("world", "2")];
    for &(key, value) in &entries {
        builder.entry(key, value);
    }

    builder.build(&mut file).unwrap();
    write!(&mut file, ";\n").unwrap();

    parse_performance_counters();

}
