use x86::cpuid;
use phf;

pub mod description;
pub mod counters;

/// Return performance counter description for the running micro-architecture.
pub fn available_counters() -> Option<&'static phf::Map<&'static str, description::IntelPerformanceCounterDescription>> {

    let cpuid = cpuid::CpuId::new();

    let vendor = match cpuid.get_vendor_info() {
        Some(vf) => String::from(vf.as_string()),
        None => String::new()
    };
    let (family, extended_model, model) = match cpuid.get_feature_info() {
        Some(fi) => (fi.family_id(), fi.extended_model_id(), fi.model_id()),
        None => (0, 0, 0)
    };

    let key = format!("{}-{}-{:X}{:X}", vendor, family, extended_model, model);

    match counters::COUNTER_MAP.contains_key(&*key) {
        true => Some(counters::COUNTER_MAP[&*key]),
        false => None
    }
}