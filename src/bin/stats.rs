extern crate perfcnt;
extern crate phf;
extern crate x86;

use x86::perfcnt::intel::counters;

fn print_stats(year: &'static str, name: &'static str, size: usize) {
    println!("{}, {}, {}", year, name, size);
}

fn main() {
    // 2008, 4
    let cc = ("Bonnell core", counters::BONNELL_CORE);
    print_stats("2008", cc.0, cc.1.len());

    // 2008, 4
    let cc = ("Nehalem EP core", counters::NEHALEMEP_CORE);
    print_stats("2008", cc.0, cc.1.len());

    let cc = ("Nehalem EX core", counters::NEHALEMEX_CORE);
    print_stats("2008", cc.0, cc.1.len());

    // 2010, 4
    let cc = ("Westmere EP DP core", counters::WESTMEREEP_DP_CORE);
    print_stats("2010", cc.0, cc.1.len());

    let cc = ("Westmere EP SP core", counters::WESTMEREEP_SP_CORE);
    print_stats("2010", cc.0, cc.1.len());

    let cc = ("Westmere EX", counters::WESTMEREEX_CORE);
    print_stats("2010", cc.0, cc.1.len());

    // 2011, 8
    let cc_uncore = ("SandyBridge uncore", counters::SANDYBRIDGE_UNCORE);
    let cc_core = ("SandyBridge core", counters::SANDYBRIDGE_CORE);
    print_stats("2011", "SandyBridge", cc_core.1.len() + cc_uncore.1.len());

    // 2011, 8
    let cc_core = ("Jaketown core", counters::JAKETOWN_CORE);
    let cc_uncore = ("Jaketown uncore", counters::JAKETOWN_UNCORE);
    print_stats("2011", "Jaketown", cc_core.1.len() + cc_uncore.1.len());

    // 2012, 8
    let cc_uncore = ("IvyBridge uncore", counters::IVYBRIDGE_UNCORE);
    let cc_core = ("IvyBridge core", counters::IVYBRIDGE_CORE);
    print_stats("2012", "IvyBridge", cc_core.1.len() + cc_uncore.1.len());

    // 2013, 8
    let cc_core = ("IvyTown core", counters::IVYTOWN_CORE);
    let cc_uncore = ("Ivytown uncore", counters::IVYTOWN_UNCORE);
    print_stats("2012", "IvyTown", cc_core.1.len() + cc_uncore.1.len());

    // 2013, 8
    let cc = ("Silvermont core", counters::SILVERMONT_CORE);
    print_stats("2013", "Silvermont", cc.1.len());

    // 2013, 8
    let cc_uncore = ("Haswell uncore", counters::HASWELL_UNCORE);
    let cc_core = ("Haswell core", counters::HASWELL_CORE);
    print_stats("2013", "Haswell", cc_core.1.len() + cc_uncore.1.len());

    // 2013, 8
    let cc_core = ("HaswellX core", counters::HASWELLX_CORE);
    let cc_uncore = ("HaswellX uncore", counters::HASWELLX_UNCORE);
    print_stats("2013", "HaswellX", cc_core.1.len() + cc_uncore.1.len());

    // 2015, 8
    let cc_core = ("Broadwell core", counters::BROADWELL_CORE);
    let cc_uncore = ("Broadwell uncore", counters::BROADWELL_UNCORE);
    print_stats("2015", "Broadwell", cc_core.1.len() + cc_uncore.1.len());

    // 2015, 8
    let cc_uncore = ("Broadwell DE uncore", counters::BROADWELLDE_UNCORE);
    let cc_core = ("Broadwell DE core", counters::BROADWELLDE_CORE);
    print_stats("2015", "Broadwell DE", cc_core.1.len() + cc_uncore.1.len());

    // 2015, 8
    let cc = ("Skylake core", counters::SKYLAKE_CORE);
    print_stats("2015", "Skylake", cc.1.len());
}
