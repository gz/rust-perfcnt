extern crate perfcnt;
extern crate x86;

use x86::perfcnt::{core_counters, uncore_counters};

fn print_counter(id: &str, info: &x86::perfcnt::intel::description::IntelPerformanceCounterDescription) {
    println!("{}:", id);

    let desc: &str = info.brief_description;
    let desc_words: Vec<&str> = desc.split(' ').collect();
    let mut chars = 0;
    print!("\t");
    for word in desc_words {
        if word.len() + chars > 60 {
            println!("");
            print!("\t");
            chars = 0;
        }
        print!("{} ", word);
        chars += word.len();
    }
    println!(" ");
    println!(" ");

}

fn main() {
    println!("All supported core performance counters on this hardware:");
    println!("----------------------------------------------------------");

    let cc = core_counters();
    let uc = uncore_counters();

    cc.map(|counters| {
        for (id, cd) in counters {
            print_counter(id, cd);
        }
    });

    println!("All supported uncore performance counters on this hardware:");
    println!("------------------------------------------------------------");
    uc.map(|counters| {
        for (id, cd) in counters {
            print_counter(id, cd);
        }
    });

    let cc_count = cc.map(|c| { c.len() } ).unwrap_or(0);
    let uc_count = uc.map(|c| { c.len() } ).unwrap_or(0);

    println!("Total H/W counters: {}", cc_count + uc_count);
}
