use nom::*;

#[derive(Debug)]
pub struct PerfFileSection {
    offset: u64,
    size: u64
}

/// Parse a perf header
named!(parse_file_section<&[u8], PerfFileSection>,
    chain!(
        offset: le_u64 ~
        size: le_u64,
        || PerfFileSection { offset: offset, size: size }
    )
);

#[derive(Debug)]
pub struct PerfFileHeader {
    size: u64,
    attr_size: u64,
    attrs: PerfFileSection,
    data: PerfFileSection,
    event_types: PerfFileSection,
    flags: Vec<u8>,
}

/// Parse a perf header
named!(parse_header<&[u8], PerfFileHeader>,
    chain!(
        tag!("PERFILE2") ~
        size: le_u64 ~
        attr_size: le_u64 ~
        attrs: parse_file_section ~
        data: parse_file_section ~
        event_types: parse_file_section ~
        flags: take!(32),
        || PerfFileHeader { size: size, attr_size: attr_size, attrs: attrs, data: data, event_types: event_types, flags: flags.to_owned() }
    )
);


pub fn parse_perf_data(buf: &[u8]) -> IResult<&[u8], PerfFileHeader> {
    parse_header(buf)
}


/*

perf_file = Struct("perf_file_header",
                   # no support for version 1
                   Magic("PERFILE2"),
                   UNInt64("size"),
                   UNInt64("attr_size"),
                   perf_file_section("attrs", perf_file_attr),
                   perf_file_section("data", perf_data),
                   perf_file_section("event_types", perf_event_types),
                   # little endian
                   Embedded(BitStruct(None,
                             Flag("nrcpus"),
                             Flag("arch"),
                             Flag("version"),
                             Flag("osrelease"),
                             Flag("hostname"),
                             Flag("build_id"),
                             Flag("tracing_data"),
                             Flag("reserved"),

                             Flag("branch_stack"),
                             Flag("numa_topology"),
                             Flag("cpu_topology"),
                             Flag("event_desc"),
                             Flag("cmdline"),
                             Flag("total_mem"),
                             Flag("cpuid"),
                             Flag("cpudesc"),

                             Padding(6),
                             Flag("group_desc"),
                             Flag("pmu_mappings"),

                             Padding(256 - 3*8))),
                   Pointer(lambda ctx: ctx.data.offset + ctx.data.size,
                           perf_features()),
                   Padding(3 * 8))

 */
