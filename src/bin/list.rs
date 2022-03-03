use x86::perfcnt::intel::{events, EventDescription};

fn print_counter(id: &str, info: &EventDescription) {
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
    println!("All supported events on this hardware:");
    println!("----------------------------------------------------------");

    let cc = events();

    cc.map(|counters| {
        for (id, cd) in counters {
            print_counter(id, cd);
        }
    });

    let cc_count = cc.map(|c| c.len()).unwrap_or(0);
    println!("Total H/W performance events: {}", cc_count);
}
