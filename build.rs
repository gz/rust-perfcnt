#![feature(convert)]

extern crate phf_codegen;
extern crate serde_json;

use std::env;
use std::fs::File;
use std::io::{BufWriter, BufReader, Write};
use std::path::Path;
use std::collections::HashMap;
use std::mem;
use std::fmt;

use serde_json::Value;

enum PebsType {
    Regular,
    PebsOrRegular,
    PebsOnly
}

impl fmt::Debug for PebsType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match *self {
            PebsType::Regular => "Regular",
            PebsType::PebsOrRegular => "PebsOrRegular",
            PebsType::PebsOnly => "PebsOnly",
        };
        write!(f, "PebsType::{}", name)
    }
}

enum EventCode {
    One(u8),
    Two(u8,u8)
}

impl fmt::Debug for EventCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EventCode::One(a) => write!(f, "EventCode::One({})", a),
            EventCode::Two(a, b) => write!(f, "EventCode::Two({}, {})", a, b),
        }
    }
}

enum MSRIndex {
    None,
    One(u8),
    Two(u8, u8)
}

impl fmt::Debug for MSRIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MSRIndex::None => write!(f, "MSRIndex::None"),
            MSRIndex::One(a) => write!(f, "MSRIndex::One({})", a),
            MSRIndex::Two(a, b) => write!(f, "MSRIndex::Two({}, {})", a, b),
        }
    }
}

enum Counter {
    /// Bit-mask containing the fixed counters
    /// usable with the corresponding performance event.
    Fixed(u8),

    /// Bit-mask containing the programmable counters
    /// usable with the corresponding performance event.
    Programmable(u8),
}

impl fmt::Debug for Counter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Counter::Fixed(a) => write!(f, "Counter::Fixed({})", a),
            Counter::Programmable(a) => write!(f, "Counter::Programmable({})", a),
        }
    }
}

#[derive(Debug)]
struct IntelPerformanceCounterDescription {

    /// This field maps to the Event Select field in the IA32_PERFEVTSELx[7:0]MSRs.
    ///
    /// The set of values for this field is defined architecturally.
    /// Each value corresponds to an event logic unit and should be used with a unit
    /// mask value to obtain an architectural performance event.
    event_code: EventCode,

    /// This field maps to the Unit Mask filed in the IA32_PERFEVTSELx[15:8] MSRs.
    ///
    /// It further qualifies the event logic unit selected in the event select
    /// field to detect a specific micro-architectural condition.
    umask: u8,

    /// It is a string of characters to identify the programming of an event.
    event_name: &'static str,

    /// This field contains a description of what is being counted by a particular event.
    brief_description: &'static str,

    /// In some cases, this field will contain a more detailed description of what is counted by an event.
    public_description: Option<&'static str>,

    /// This field lists the fixed (PERF_FIXED_CTRX) or programmable (IA32_PMCX)
    /// counters that can be used to count the event.
    counter: Counter,

    /// This field lists the counters where this event can be sampled
    /// when Intel® Hyper-Threading Technology (Intel® HT Technology) is
    /// disabled.
    ///
    /// When Intel® HT Technology is disabled, some processor cores gain access to
    /// the programmable counters of the second thread, making a total of eight
    /// programmable counters available. The additional counters will be
    /// numbered 4,5,6,7. Fixed counter behavior remains unaffected.
    counter_ht_off: Counter,

    /// This field is only relevant to PEBS events.
    ///
    /// It lists the counters where the event can be sampled when it is programmed as a PEBS event.
    pebs_counters: Option<Counter>,

    /// Sample After Value (SAV) is the value that can be preloaded
    /// into the counter registers to set the point at which they will overflow.
    ///
    /// To make the counter overflow after N occurrences of the event,
    /// it should be loaded with (0xFF..FF – N) or –(N-1). On overflow a
    /// hardware interrupt is generated through the Local APIC and additional
    /// architectural state can be collected in the interrupt handler.
    /// This is useful in event-based sampling. This field gives a recommended
    /// default overflow value, which may be adjusted based on workload or tool preference.
    sample_after_value: u64,

    /// Additional MSRs may be required for programming certain events.
    /// This field gives the address of such MSRS.
    msr_index: MSRIndex,

    /// When an MSRIndex is used (indicated by the MSRIndex column), this field will
    /// contain the value that needs to be loaded into the
    /// register whose address is given in MSRIndex column.
    ///
    /// For example, in the case of the load latency events, MSRValue defines the
    /// latency threshold value to write into the MSR defined in MSRIndex (0x3F6).
    msr_value: u64,

    /// This field is set for an event which can only be sampled or counted by itself,
    /// meaning that when this event is being collected,
    /// the remaining programmable counters are not available to count any other events.
    taken_alone: bool,

    /// This field maps to the Counter Mask (CMASK) field in IA32_PERFEVTSELx[31:24] MSR.
    counter_mask: u8,

    /// This field corresponds to the Invert Counter Mask (INV) field in IA32_PERFEVTSELx[23] MSR.
    invert: bool,

    /// This field corresponds to the Any Thread (ANY) bit of IA32_PERFEVTSELx[21] MSR.
    any_thread: bool,

    /// This field corresponds to the Edge Detect (E) bit of IA32_PERFEVTSELx[18] MSR.
    edge_detect: bool,

    /// A '0' in this field means that the event cannot be programmed as a PEBS event.
    /// A '1' in this field means that the event is a  precise event and can be programmed
    /// in one of two ways – as a regular event or as a PEBS event.
    /// And a '2' in this field means that the event can only be programmed as a PEBS event.
    pebs: PebsType,

    /// A '1' in this field means the event uses the Precise Store feature and Bit 3 and
    /// bit 63 in IA32_PEBS_ENABLE MSR must be set to enable IA32_PMC3 as a PEBS counter
    /// and enable the precise store facility respectively.
    ///
    /// Processors based on SandyBridge and IvyBridge micro-architecture offer a
    /// precise store capability that provides a means to profile store memory
    /// references in the system.
    precise_store: bool,

    /// A '1' in this field means that when the event is configured as a PEBS event,
    /// the Data Linear Address facility is supported.
    ///
    /// The Data Linear Address facility is a new feature added to Haswell as a
    /// replacement or extension of the precise store facility in SNB.
    data_la: bool,

    /// A '1' in this field means that when the event is configured as a PEBS event,
    /// the DCU hit field of the PEBS record is set to 1 when the store hits in the
    /// L1 cache and 0 when it misses.
    l1_hit_indication: bool,

    /// This field lists the known bugs that apply to the events.
    ///
    /// For the latest, up to date errata refer to the following links:
    ////
    /// * Haswell:
    ///   http://www.intel.com/content/dam/www/public/us/en/documents/specification-updates/4th-gen-core-family-mobile-specification-update.pdf
    ///
    /// * IvyBridge:
    ///   https://www-ssl.intel.com/content/dam/www/public/us/en/documents/specification-updates/3rd-gen-core-desktop-specification-update.pdf
    ///
    /// * SandyBridge:
    ///   https://www-ssl.intel.com/content/dam/www/public/us/en/documents/specification-updates/2nd-gen-core-family-mobile-specification-update.pdf
    errata: Option<&'static str>,

    /// There is only 1 file for core and offcore events in this format.
    /// This field is set to 1 for offcore events and 0 for core events.
    offcore: bool,
}

impl IntelPerformanceCounterDescription {

    fn new(event_code: EventCode, umask: u8, event_name: &'static str,
           brief_description: &'static str, public_description: Option<&'static str>,
           counter: Counter, counter_ht_off: Counter, pebs_counters: Option<Counter>,
           sample_after_value: u64, msr_index: MSRIndex, msr_value: u64, taken_alone: bool,
           counter_mask: u8, invert: bool, any_thread: bool, edge_detect: bool, pebs:
           PebsType, precise_store: bool, data_la: bool, l1_hit_indication: bool,
           errata: Option<&'static str>, offcore: bool) -> IntelPerformanceCounterDescription {

        IntelPerformanceCounterDescription {
            event_code: event_code,
            umask: umask,
            event_name: event_name,
            brief_description: brief_description,
            public_description: public_description,
            counter: counter,
            counter_ht_off: counter_ht_off,
            pebs_counters: pebs_counters,
            sample_after_value: sample_after_value,
            msr_index: msr_index,
            msr_value: msr_value,
            taken_alone: taken_alone,
            counter_mask: counter_mask,
            invert: invert,
            any_thread: any_thread,
            edge_detect: edge_detect,
            pebs: pebs,
            precise_store: precise_store,
            data_la: data_la,
            l1_hit_indication: l1_hit_indication,
            errata: errata,
            offcore: offcore
        }

    }
}


/// We need to convert parsed strings to static because we're reusing
/// the struct definition which declare strings (rightgully) as
/// static in the generated code.
fn string_to_static_str<'a>(s: &'a str) -> &'static str {
    unsafe {
        let ret = mem::transmute(&s as &str);
        mem::forget(s);
        ret
    }
}

fn parse_performance_counters(input: &str) {
    let mut builder = phf_codegen::Map::new();
    let f = File::open(input).unwrap();
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
            let mut offcore = false;

            let mut all_events = HashMap::new();

            let pcn = entry.as_object().unwrap();
            for (key, value) in pcn.iter() {
                if !value.is_string() {
                    panic!("Not a string");
                }
                let value_string = value.as_string().unwrap();
                let value_str = string_to_static_str(value_string);
                let split_str_parts: Vec<&str> = value_string.split(", ").collect();

                match key.as_str() {
                    "EventName" => {
                        if !all_events.contains_key(value_str) {
                            all_events.insert(value_str, 0);
                        }
                        else {
                            panic!("Key {} already exists.", value_str);
                        }
                        event_name = value_str;
                    }

                    "EventCode" => {
                        let split_parts: Vec<u64> = split_str_parts.iter()
                            .map(|x| { assert!(x.starts_with("0x")); u64::from_str_radix(&x[2..], 16).unwrap() })
                            .collect();

                        match split_parts.len() {
                            1 => event_code = EventCode::One(split_parts[0] as u8),
                            2 => event_code = EventCode::Two(split_parts[0] as u8, split_parts[1] as u8),
                            _ => panic!("More than two event codes?")
                        }
                    },

                    "UMask" => {
                        assert!(value_str[..2].starts_with("0x"));
                        umask = u64::from_str_radix(&value_str[2..], 16).unwrap() as u8
                    },

                    "BriefDescription" => brief_description = value_str,

                    "PublicDescription" => {
                        if brief_description != value_str && value_str != "tbd" {
                            public_description = Some(value_str);
                        }
                        else {
                            public_description = None;
                        }
                    },

                    "Counter" => {
                        if value_str.starts_with("Fixed counter") {
                            let mask: u64 = value_str["Fixed counter".len()..]
                                .split(",")
                                .map(|x| x.trim())
                                .map(|x| u64::from_str_radix(&x, 10).unwrap())
                                .fold(0, |acc, c| { assert!(c < 8); acc | 1 << c });
                            counter = Counter::Fixed(mask as u8);
                        }
                        else {
                            let mask: u64 = value_str
                                .split(",")
                                .map(|x| x.trim())
                                .map(|x| u64::from_str_radix(&x, 10).unwrap())
                                .fold(0, |acc, c| { assert!(c < 8); acc | 1 << c });
                            counter = Counter::Programmable(mask as u8);
                        }
                    },

                    "CounterHTOff" => {
                        if value_str.starts_with("Fixed counter") {
                            let mask: u64 = value_str["Fixed counter".len()..]
                                .split(",")
                                .map(|x| x.trim())
                                .map(|x| u64::from_str_radix(&x, 10).unwrap())
                                .fold(0, |acc, c| { assert!(c < 8); acc | 1 << c });
                            counter_ht_off = Counter::Fixed(mask as u8);
                        }
                        else {
                            let mask: u64 = value_str
                                .split(",")
                                .map(|x| x.trim())
                                .map(|x| u64::from_str_radix(&x, 10).unwrap())
                                .fold(0, |acc, c| { assert!(c < 8); acc | 1 << c });
                            counter_ht_off = Counter::Programmable(mask as u8);
                        }
                    },

                    "PEBScounters" => {
                        if value_str.starts_with("Fixed counter") {
                            let mask: u64 = value_str["Fixed counter".len()..]
                                .split(",")
                                .map(|x| x.trim())
                                .map(|x| u64::from_str_radix(&x, 10).unwrap())
                                .fold(0, |acc, c| { assert!(c < 8); acc | 1 << c });
                            pebs_counters = Some(Counter::Fixed(mask as u8));
                        }
                        else {
                            let mask: u64 = value_str
                                .split(",")
                                .map(|x| x.trim())
                                .map(|x| u64::from_str_radix(&x, 10).unwrap())
                                .fold(0, |acc, c| { assert!(c < 8); acc | 1 << c });
                            pebs_counters = Some(Counter::Programmable(mask as u8));
                        }
                    },

                    "SampleAfterValue" => sample_after_value = u64::from_str_radix(&value_str, 10).unwrap(),

                    "MSRIndex" => {
                        let split_parts: Vec<u64> = value_str
                            .split(",")
                            .map(|x| x.trim())
                            .map(|x| {
                                if x.len() > 2 && x[..2].starts_with("0x") {
                                    u64::from_str_radix(&x[2..], 16).unwrap()
                                }
                                else {
                                    u64::from_str_radix(&x, 10).unwrap()
                                }
                            })
                            .collect();

                            msr_index = match split_parts.len() {
                                1 => {
                                    if split_parts[0] != 0 {
                                        MSRIndex::One(split_parts[0] as u8)
                                    }
                                    else {
                                        MSRIndex::None
                                    }
                                },
                                2 => MSRIndex::Two(split_parts[0] as u8, split_parts[1] as u8),
                                _ => panic!("More than two MSR indexes?")
                            }
                    },
                    "MSRValue" => {
                        msr_value = if value_str.len() > 2 && value_str[..2].starts_with("0x") {
                            u64::from_str_radix(&value_str[2..], 16).unwrap()
                        }
                        else {
                            u64::from_str_radix(&value_str, 10).unwrap()
                        }
                    },
                    "TakenAlone" => {
                        taken_alone = match value_str.trim() {
                            "0" => false,
                            "1" => true,
                            _ => panic!("Unknown boolean value {}", value_str),
                        };
                    },
                    "CounterMask" => {
                        counter_mask = if value_str.len() > 2 && value_str[..2].starts_with("0x") {
                            u8::from_str_radix(&value_str[2..], 16).unwrap()
                        }
                        else {
                            u8::from_str_radix(&value_str, 10).unwrap()
                        }
                    },
                    "Invert" => {
                        invert = match value_str.trim() {
                            "0" => false,
                            "1" => true,
                            _ => panic!("Unknown boolean value {}", value_str),
                        };
                    }
                    "AnyThread" => any_thread = match value_str.trim() {
                            "0" => false,
                            "1" => true,
                            _ => panic!("Unknown boolean value {}", value_str),
                        },
                    "EdgeDetect" => edge_detect = match value_str.trim() {
                            "0" => false,
                            "1" => true,
                            _ => panic!("Unknown boolean value {}", value_str),
                        },
                    "PEBS" => {
                        pebs = match value_str.trim() {
                            "0" => PebsType::Regular,
                            "1" => PebsType::PebsOrRegular,
                            "2" => PebsType::PebsOnly,
                            _ => panic!("Unknown PEBS type: {}", value_str),
                        }
                    },
                    "PreciseStore" => precise_store = match value_str.trim() {
                            "0" => false,
                            "1" => true,
                            _ => panic!("Unknown boolean value {}", value_str),
                        },
                    "Data_LA" => data_la = match value_str.trim() {
                            "0" => false,
                            "1" => true,
                            _ => panic!("Unknown boolean value {}", value_str),
                        },
                    "L1_Hit_Indication" => l1_hit_indication = match value_str.trim() {
                            "0" => false,
                            "1" => true,
                            _ => panic!("Unknown boolean value {}", value_str),
                        },
                    "Errata" => {
                        errata = if value_str != "null" {
                            Some(value_str)
                        }
                        else {
                            None
                        };
                    },
                    "Offcore" => offcore = match value_str.trim() {
                            "0" => false,
                            "1" => true,
                            _ => panic!("Unknown boolean value {}", value_str),
                        },
                    _ => panic!("Unknown member: {}", key),
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

            println!("{:?}", ipcd.event_name);
            builder.entry(ipcd.event_name, format!("{:?}", ipcd).as_str());

            //println!("{:?}", ipcd);
        }
    }
    else {
        panic!("JSON data is not an array.");
    }

    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());
    write!(&mut file, "static PERFORMANCE_COUNTER_HASWELL: phf::Map<&'static str, IntelPerformanceCounterDescription> = ").unwrap();
    builder.build(&mut file).unwrap();
    write!(&mut file, ";\n").unwrap();
}

fn main() {
    parse_performance_counters("Haswell_core_V20.json");
}
