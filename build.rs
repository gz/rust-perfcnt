extern crate phf_codegen;
extern crate serde_json;

use std::env;
use std::fs::File;
use std::io::{BufWriter, BufReader, Write};
use std::path::Path;

use serde_json::Value;

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/intel/mod.rs"));

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

            let mut event_code = EventCode::One(0);
            let mut umask = 0;
            let mut event_name = "";
            let mut brief_description = "";
            let mut public_description = None;
            let mut counter = Counter::Fixed(0);
            let mut counter_ht_off = Counter::Fixed(0);
            let mut pebs_counters = None;
            let mut sample_after_value = 0;
            let mut msr_index = MSRIndex::None;
            let mut msr_value = 0;
            let mut taken_alone = false;
            let mut counter_mask = 0;
            let mut invert = false;
            let mut any_thread = false;
            let mut edge_detect = false;
            let mut pebs = PebsType::Regular;
            let mut precise_store = false;
            let mut data_la = false;
            let mut l1_hit_indication = false;
            let mut errata = None;
            let offcore = false;

            let pcn = entry.as_object().unwrap();
            for (key, value) in pcn.iter() {

                match key {
                    "EventCode" => event_code = EventCode::One(0x1),
                };

            }

            let ipcd = IntelPerformanceCounterDescription::new(
                event_code,
                umask,
                event_name,
                brief_description,
                public_description,
                counter,
                counter_ht_off,
                pebs_counters,
                sample_after_value,
                msr_index,
                msr_value,
                taken_alone,
                counter_mask,
                invert,
                any_thread,
                edge_detect,
                pebs,
                precise_store,
                data_la,
                l1_hit_indication,
                errata,
                offcore
            );

            println!("{:?}", ipcd);
        }
    }
    else {
        panic!("JSON data is not an array.");
    }

    panic!("done");
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
