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

#[derive(Debug)]
struct ThreadId {
    pid: i32,
    tid: i32
}

#[derive(Debug)]
struct Cpu {
    cpu: u32,
    res: u32
}

struct Callchain {

}


#[derive(Debug, Eq, PartialEq)]
enum EventType {
    Mmap,
    Lost,
    Comm,
    Exit,
    Throttle,
    Unthrottle,
    Fork,
    Read,
    Sample,
    Mmap2
}

impl EventType {
    fn new(event_type: u32) -> EventType {
        match event_type {
            1 => EventType::Mmap,
            2 => EventType::Lost,
            3 => EventType::Comm,
            4 => EventType::Exit,
            5 => EventType::Throttle,
            6 => EventType::Unthrottle,
            7 => EventType::Fork,
            8 => EventType::Read,
            9 => EventType::Sample,
            10 => EventType::Mmap2,
            _ => panic!("Unknown event type ({}) encountered, update the enum?", event_type)
        }
    }
}

#[derive(Debug)]
struct EventHeader {
    event_type: EventType,
    misc: u16,
    size: u16,
}

impl EventHeader {
    pub fn size(&self) -> usize {
        self.size as usize
    }
}

/// Parse a file section
named!(parse_event_header<&[u8], EventHeader>,
    chain!(
        event_type: le_u32 ~
        misc: le_u16 ~
        size: le_u16,
        || EventHeader { event_type: EventType::new(event_type), misc: misc, size: size }
    )
);

/// The MMAP events record the PROT_EXEC mappings so that we can correlate user-space IPs to code.
#[derive(Debug)]
pub struct MMAPRecord {
    pid: i32,
    tid: u32,
    addr: u64,
    len: u64,
    pgoff: u64,
    filename: String
}

#[derive(Default, Debug)]
pub struct FileReadFormat {
    /// The value of the event
    pub value: u64,
    /// if PERF_FORMAT_TOTAL_TIME_ENABLED
    pub time_enabled: u64,
    /// if PERF_FORMAT_TOTAL_TIME_RUNNING
    pub time_running: u64,
    /// if PERF_FORMAT_ID
    pub id: u64,
}

#[derive(Debug)]
pub struct ReadRecord {
    pid: u32,
    tid: u32,
    value: FileReadFormat, // TODO with PERF_FORMAT_GROUP: values: Vec<FileReadFormat>
}

#[derive(Debug)]
struct BranchEntry {
    pub from: u64,
    pub to: u64,
    flags: u64,
}

/// This record indicates a sample.
#[derive(Debug)]
pub struct SampleRecord {
    /// if PERF_SAMPLE_IP
    ip: Option<u64>,
    /// if PERF_SAMPLE_TID
    pid_tid: Option<ThreadId>,
    /// if PERF_SAMPLE_TIME
    time: Option<u64>,
    /// if PERF_SAMPLE_ADDR
    addr: Option<u64>,
    /// if PERF_SAMPLE_ID
    id: Option<u64>,
    /// if PERF_SAMPLE_STREAM_ID
    stream_id: Option<u64>,
    /// if PERF_SAMPLE_CPU
    cpu_res: Option<Cpu>,
    /// if PERF_SAMPLE_PERIOD
    period: Option<u64>,
    /// if PERF_SAMPLE_READ
    v: FileReadFormat, // # TODO FILE GROUP FORMAT is different...
    /// if PERF_SAMPLE_CALLCHAIN
    ips: Option<Vec<u64>>,
    /// if PERF_SAMPLE_RAW
    raw_sample: Option<Vec<u8>>,
    /// if PERF_SAMPLE_REGS_USER & PERF_SAMPLE_BRANCH_STACK
    lbr: Option<Vec<BranchEntry>>,
    /// PERF_SAMPLE_STACK_USER
    abi_user: Option<u64>,
    /// PERF_SAMPLE_STACK_USER
    regs_user: Option<Vec<u64>>,
    /// PERF_SAMPLE_STACK_USER
    user_stack: Option<Vec<u8>>,
    /// PERF_SAMPLE_STACK_USER
    dyn_size: Option<u64>,
    /// if PERF_SAMPLE_WEIGHT
    weight: Option<u64>,
    /// if PERF_SAMPLE_DATA_SRC
    data_src: Option<u64>,
    /// if PERF_SAMPLE_TRANSACTION
    transaction: Option<u64>,
    /// if PERF_SAMPLE_REGS_INTR
    abi_intr: Option<u64>,
    /// if PERF_SAMPLE_REGS_INTR
    regs_intr: Option<Vec<u64>>
}

#[derive(Debug)]
pub struct LostRecord {

}

#[derive(Debug)]
struct Event {
    header: EventHeader,
    data: EventData,
}

#[derive(Debug)]
pub enum EventData {
    MMAP(MMAPRecord),
    Lost(LostRecord),
    //Comm(CommRecord),
    //Exit(ExitRecord),
    //Throttle(ThrottleRecord),
    //Unthrottle(ThrottleRecord),
    //Fork(ForkRecord),
    //Read(ReadRecord),
    //Sample(SampleRecord),
}

fn is_nul_byte(c: &u8) -> bool {
    *c == 0x0
}

named!(parse_c_string, take_till!(is_nul_byte));

named!(parse_mmap_event<&[u8], EventData>,
    chain!(
        pid: le_i32 ~
        tid: le_u32 ~
        addr: le_u64 ~
        len: le_u64 ~
        pgoff: le_u64 ~
        filename: parse_c_string,
        || EventData::MMAP(MMAPRecord {
                pid: pid,
                tid: tid,
                addr: addr,
                len: len,
                pgoff: pgoff,
                filename: unsafe { String::from_utf8_unchecked(filename.to_vec()) }
        })
    )
);

/// Parse a file section
named!(parse_event<&[u8], Event>,
    chain!(
        header: parse_event_header ~
        event: alt!(
            cond_reduce!(header.event_type == EventType::Mmap, parse_mmap_event) |
            cond_reduce!(header.event_type == EventType::Lost, parse_mmap_event)
        ),
        || Event { header: header, data: event })
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

impl PerfFileSection {
    fn start(&self) -> usize {
        self.offset as usize
    }

    fn end(&self) -> usize {
        (self.offset + self.size) as usize
    }
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
        let mut slice: &[u8] = &self.bytes[self.header.data.start()..self.header.data.end()];

        while slice.len() > 0 {
            let r = parse_event(slice);
            match r {
                IResult::Done(rest, ev) => {
                    println!("{:?}", ev);
                    println!("Parsed bytes: {:?}", slice.len() - rest.len());
                    println!("Event size: {:?}", ev.header.size());
                    println!("Padding: {:?}", ev.header.size() - (slice.len() - rest.len()) );
                    slice = rest.split_at( ev.header.size() - (slice.len() - rest.len()) ).1;
                },
                _ => break
            }
        }
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
