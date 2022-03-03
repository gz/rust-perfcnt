use x86::perfcnt::intel::events;

fn print_stats(year: &'static str, name: &'static str, size: usize) {
    println!("{}, {}, {}", year, name, size);
}

fn main() {
    // 2008, 4
    let cc = ("Bonnell core", events::BONNELL);
    print_stats("2008", cc.0, cc.1.len());

    // 2008, 4
    let cc = ("Nehalem EP core", events::NEHALEMEP);
    print_stats("2008", cc.0, cc.1.len());

    let cc = ("Nehalem EX core", events::NEHALEMEX);
    print_stats("2008", cc.0, cc.1.len());

    // 2010, 4
    let cc = ("Westmere EP DP core", events::WESTMEREEP_DP);
    print_stats("2010", cc.0, cc.1.len());

    let cc = ("Westmere EP SP core", events::WESTMEREEP_SP);
    print_stats("2010", cc.0, cc.1.len());

    let cc = ("Westmere EX", events::WESTMEREEX);
    print_stats("2010", cc.0, cc.1.len());

    // 2011, 8
    let cc = ("SandyBridge core", events::SANDYBRIDGE);
    print_stats("2011", "SandyBridge", cc.1.len());

    // 2011, 8
    let cc = ("Jaketown core", events::JAKETOWN);
    print_stats("2011", "Jaketown", cc.1.len());

    // 2012, 8
    let cc = ("IvyBridge core", events::IVYBRIDGE);
    print_stats("2012", "IvyBridge", cc.1.len());

    // 2013, 8
    let cc = ("IvyTown core", events::IVYTOWN);
    print_stats("2012", "IvyTown", cc.1.len());

    // 2013, 8
    let cc = ("Silvermont core", events::SILVERMONT);
    print_stats("2013", "Silvermont", cc.1.len());

    // 2013, 8
    let cc = ("Haswell core", events::HASWELL);
    print_stats("2013", "Haswell", cc.1.len());

    // 2013, 8
    let cc = ("HaswellX core", events::HASWELLX);
    print_stats("2013", "HaswellX", cc.1.len());

    // 2015, 8
    let cc = ("Broadwell core", events::BROADWELL);
    print_stats("2015", "Broadwell", cc.1.len());

    // 2015, 8
    print_stats("2015", "Broadwell DE", cc.1.len());

    // 2015, 8
    let cc = ("Skylake core", events::SKYLAKE);
    print_stats("2015", "Skylake", cc.1.len());
}
