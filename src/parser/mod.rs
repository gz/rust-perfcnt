use nom::*;

use std::iter::Zip;
use std::slice::Iter;
use std::vec::IntoIter;

#[derive(Debug)]
struct ForkExit {
    pid: i32,
    ppid: i32,
    tid: i32,
    ptid: i32,
    time: i32
    // sample id?
}

struct Throttle {
    time: u64,
    id: u64,
    stream_id: u64
    // sample id?
}

struct ThreadId {
    pid: i32,
    tid: i32
}

struct Cpu {
    cpu: u32,
    res: u32
}

struct Callchain {

}

struct Event {
    identifier: u64,
    ip: u64,
    tid: ThreadId,
    time: u64,
    add: u64,
    id: u64,
    stream_id: u64,
    cpu: Cpu,
    period: u64,
    read: u8, // XXX
    caller: Vec<u64>,

}

enum EventType {
    Mmap,
    Lost,
    Comm,
    Throttle,
    Unthrottle,
    Fork,
    Read,
    Sample,
    Mmap2,
    TracingData,
    FinishedRound,
    IdIndex,
    AuxtraceInfo,
    Auxtrace,
    AuxtraceError
}

#[derive(Debug)]
struct EventHeader {
    event_type: u32,
    flags: u16,
    size: u16,

}

/// Parse a file section
named!(parse_event_header<&[u8], EventHeader>,
    chain!(
        event_type: le_u32 ~
        flags: le_u16 ~
        size: le_u16,
        || EventHeader { event_type: event_type, flags: flags, size: size }
    )
);


/*
def perf_event_header():
    return Embedded(Struct(None,
                           Enum(UNInt32("type"),
                                MMAP			= 1,
                                LOST			= 2,
                                COMM			= 3,
                                EXIT			= 4,
                                THROTTLE		= 5,
                                UNTHROTTLE		= 6,
                                FORK			= 7,
                                READ			= 8,
                                SAMPLE			= 9,
                                MMAP2			= 10,
                                TRACING_DATA            = 66,
                                FINISHED_ROUND          = 68,
                                ID_INDEX                = 69,
                                AUXTRACE_INFO           = 70,
                                AUXTRACE                = 71,
                                AUXTRACE_ERROR          = 72),
                           Embedded(BitStruct(None,
                                              Padding(1),
                                              Enum(BitField("cpumode", 7),
                                                   UNKNOWN = 0,
                                                   KERNEL = 1,
                                                   USER = 2,
                                                   HYPERVISOR = 3,
                                                   GUEST_KERNEL = 4,
                                                   GUEST_USER = 5),

                                              Flag("ext_reserved"),
                                              Flag("exact_ip"),
                                              Flag("mmap_data"),
                                              Padding(5))),
                           UNInt16("size"),
                           If(has_sample_id_all,
                                 Pointer(lambda ctx: ctx.start + ctx.size - 8,
                                   UNInt64("end_id"))),
                           Value("attr", lookup_event_attr)))
*/

#[derive(Debug, Clone, Copy)]
pub struct PerfFileSection {
    offset: u64,
    size: u64
}

/// Parse a file section
named!(parse_file_section<&[u8], PerfFileSection>,
    chain!(
        offset: le_u64 ~
        size: le_u64,
        || PerfFileSection { offset: offset, size: size }
    )
);

#[derive(Debug, Clone, Copy)]
pub enum HeaderFlag {
    NrCpus,
    Arch,
    Version,
    OsRelease,
    Hostname,
    BuildId,
    TracingData,
    BranchStack,
    NumaTopology,
    CpuTopology,
    EventDesc,
    CmdLine,
    TotalMem,
    CpuId,
    CpuDesc,
    GroupDesc,
    PmuMappings
}

#[derive(Debug)]
pub struct HeaderFlags {
    nrcpus: bool,
    arch: bool,
    version: bool,
    osrelease: bool,
    hostname: bool,
    build_id: bool,
    tracing_data: bool,
    branch_stack: bool,
    numa_topology: bool,
    cpu_topology: bool,
    event_desc: bool,
    cmdline: bool,
    total_mem: bool,
    cpuid: bool,
    cpudesc: bool,
    group_desc: bool,
    pmu_mappings: bool,
}

impl HeaderFlags {
    fn collect(&self) -> Vec<HeaderFlag> {
        let mut flags = Vec::with_capacity(17);

        if self.nrcpus {
            flags.push(HeaderFlag::NrCpus);
        }
        if self.arch {
            flags.push(HeaderFlag::Arch);
        }
        if self.version {
            flags.push(HeaderFlag::Version);
        }
        if self.osrelease {
            flags.push(HeaderFlag::OsRelease);
        }
        if self.hostname {
            flags.push(HeaderFlag::Hostname);
        }
        if self.build_id {
            flags.push(HeaderFlag::BuildId);
        }
        if self.tracing_data {
            flags.push(HeaderFlag::TracingData);
        }
        if self.branch_stack {
            flags.push(HeaderFlag::BranchStack);
        }
        if self.numa_topology {
            flags.push(HeaderFlag::NumaTopology);
        }
        if self.cpu_topology {
            flags.push(HeaderFlag::CpuTopology);
        }
        if self.event_desc {
            flags.push(HeaderFlag::EventDesc);
        }
        if self.cmdline {
            flags.push(HeaderFlag::CmdLine);
        }
        if self.total_mem {
            flags.push(HeaderFlag::TotalMem);
        }
        if self.cpuid {
            flags.push(HeaderFlag::CpuId);
        }
        if self.cpudesc {
            flags.push(HeaderFlag::CpuDesc);
        }
        if self.group_desc {
            flags.push(HeaderFlag::GroupDesc);
        }
        if self.pmu_mappings {
            flags.push(HeaderFlag::PmuMappings);
        }

        flags
    }
}

#[derive(Debug)]
pub struct PerfFileHeader {
    size: u64,
    attr_size: u64,
    attrs: PerfFileSection,
    data: PerfFileSection,
    event_types: PerfFileSection,

    flags: HeaderFlags,
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
        flags: bits!(chain!(
            nrcpus: take_bits!(u8, 1) ~
            arch: take_bits!(u8, 1) ~
            version: take_bits!(u8, 1) ~
            osrelease: take_bits!(u8, 1) ~
            hostname: take_bits!(u8, 1) ~
            build_id: take_bits!(u8, 1) ~
            tracing_data: take_bits!(u8, 1) ~
            reserved: take_bits!(u8, 1) ~

            branch_stack: take_bits!(u8, 1) ~
            numa_topology: take_bits!(u8, 1) ~
            cpu_topology: take_bits!(u8, 1) ~
            event_desc: take_bits!(u8, 1) ~
            cmdline: take_bits!(u8, 1) ~
            total_mem: take_bits!(u8, 1) ~
            cpuid: take_bits!(u8, 1) ~
            cpudesc: take_bits!(u8, 1) ~

            pad1: take_bits!(u8, 6) ~
            group_desc: take_bits!(u8, 1) ~
            pmu_mappings: take_bits!(u8, 1),
            || {
                HeaderFlags {
                    nrcpus: nrcpus == 1,
                    arch: arch == 1,
                    version: version == 1,
                    osrelease: osrelease == 1,
                    hostname: hostname == 1,
                    build_id: build_id == 1,
                    tracing_data: tracing_data == 1,
                    branch_stack: branch_stack == 1,
                    numa_topology: numa_topology == 1,
                    cpu_topology: cpu_topology == 1,
                    event_desc: event_desc == 1,
                    cmdline: cmdline == 1,
                    total_mem: total_mem == 1,
                    cpuid: cpuid == 1,
                    cpudesc: cpudesc == 1,
                    group_desc: group_desc == 1,
                    pmu_mappings: pmu_mappings == 1
                }
            }
        )) ~
        reserved: take!(29),
        || PerfFileHeader { size: size, attr_size: attr_size, attrs: attrs, data: data, event_types: event_types, flags: flags }
    )
);

#[derive(Debug)]
pub struct PerfFile {
    bytes: Vec<u8>,
    pub header: PerfFileHeader,
    //sections: Vec<PerfFileSection>,
}

impl PerfFile {
    pub fn new(bytes: Vec<u8>) -> PerfFile {
        let header = match parse_header(bytes.as_slice()) {
            IResult::Done(rest, h) => h,
            IResult::Error(e) => panic!("{:?}", e),
            IResult::Incomplete(_) => panic!("Incomplete data?"),
        };

        PerfFile { header: header, bytes: bytes }
    }

    pub fn data(&self) {
        let data_start = self.header.data.offset as usize;
        let slice: &[u8] = &self.bytes[data_start..];
        println!("{:?}", parse_event_header(slice));
    }

    pub fn sections(&self) -> Vec<(HeaderFlag, PerfFileSection)> {
        let sections: Vec<PerfFileSection> = self.parse_header_sections().unwrap().1;
        let flags: Vec<HeaderFlag> = self.header.flags.collect();
        assert!(sections.len() == flags.len());

        flags.into_iter().zip(sections).collect()
    }

    fn parse_header_sections(&self) -> IResult<&[u8], Vec<PerfFileSection>> {
        let sections_start: usize = (self.header.data.offset + self.header.data.size) as usize;
        let slice: &[u8] = &self.bytes[sections_start..];
        let flags: Vec<HeaderFlag> = self.header.flags.collect();

        count!(slice, parse_file_section, flags.len())
    }
}
