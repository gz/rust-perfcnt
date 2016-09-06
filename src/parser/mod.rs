use nom::*;

#[derive(Debug)]
struct ThreadId {
    pid: i32,
    tid: i32
}

named!(parse_thread_id<&[u8], ThreadId>,
    chain!(
        pid: le_i32 ~
        tid: le_i32,
        || ThreadId { pid: pid, tid: tid }
    )
);

/*
TODO:

#[derive(Debug)]
struct SampleId {
    /// if PERF_SAMPLE_TID set
    ptid: ThreadId,
    /// if PERF_SAMPLE_TIME set
    time: u64,
    /// if PERF_SAMPLE_ID set
    id: u64,
    /// if PERF_SAMPLE_STREAM_ID set
    stream_id: u64,
    /// if PERF_SAMPLE_CPU set
    cpu: Cpu,
    /// if PERF_SAMPLE_IDENTIFIER set
    identifier: u64
}

// TODO: Needs sample flags!
named!(parse_sample_id<&[u8], SampleId>,
    chain!(
        ptid: parse_thread_id ~
        time: le_u64 ~
        id: le_u64 ~
        stream_id: le_u64 ~
        cpu: parse_cpu ~
        identifier: le_u64,
        || SampleId {
            ptid: ptid,
            time: time,
            id: id,
            stream_id: stream_id,
            cpu: cpu,
            identifier: identifier
        }
    )
);
*/

#[derive(Debug)]
struct Cpu {
    cpu: u32,
    res: u32
}

named!(parse_cpu<&[u8], Cpu>,
    chain!(
        cpu: le_u32 ~
        res: le_u32,
        || Cpu { cpu: cpu, res: res }
    )
);

/// This record indicates a fork event.
#[derive(Debug)]
pub struct ForkRecord {
    pid: u32,
    ppid: u32,
    tid: u32,
    ptid: u32,
    time: u64,
    // TOOD: sample_id
}

named!(parse_fork_event<&[u8], EventData>,
    chain!(
        pid: le_u32 ~
        ppid: le_u32 ~
        tid: le_u32 ~
        ptid: le_u32 ~
        time: le_u64,
        || EventData::Fork(ForkRecord {
            pid: pid,
            ppid: ppid,
            tid: tid,
            ptid: ptid,
            time: time,
        })
    )
);


/// This record indicates a process exit event.
#[derive(Debug)]
pub struct ExitRecord {
    pid: u32,
    ppid: u32,
    tid: u32,
    ptid: u32,
    time: u64
    // TOOD: sample_id
}

named!(parse_exit_event<&[u8], EventData>,
    chain!(
        pid: le_u32 ~
        ppid: le_u32 ~
        tid: le_u32 ~
        ptid: le_u32 ~
        time: le_u64,
        || EventData::Exit(ExitRecord {
            pid: pid,
            ppid: ppid,
            tid: tid,
            ptid: ptid,
            time: time,
        })
    )
);

#[derive(Debug)]
pub struct ThrottleRecord {
    time: u64,
    id: u64,
    stream_id: u64
    // TODO: sample id?
}

named!(parse_throttle_event<&[u8], EventData>,
    chain!(
        time: le_u64 ~
        id: le_u64 ~
        stream_id: le_u64,
        || EventData::Throttle(ThrottleRecord {
            time: time,
            id: id,
            stream_id: stream_id,
        })
    )
);

#[derive(Debug)]
pub struct UnthrottleRecord {
    time: u64,
    id: u64,
    stream_id: u64
    // TODO: sample id?
}

named!(parse_unthrottle_event<&[u8], EventData>,
    chain!(
        time: le_u64 ~
        id: le_u64 ~
        stream_id: le_u64,
        || EventData::Unthrottle(UnthrottleRecord {
            time: time,
            id: id,
            stream_id: stream_id,
        })
    )
);

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
    Mmap2,
    // Aux, // 11
    // ITraceStart, // 12
    // LostSamples, // 13
    // Switch, // 14
    // SwitchCpuWide, // 15
    // HeaderAttr, // 64
    // HeaderEventType, // 65, deprecated
    // HeaderTracingData, // 66
    // HeaderBuildId, // 67
    FinishedRound, // 68
    // RecordIdIndex, // 69
    // AuxTraceInfo, // 70
    // AuxTrace, // 71
    // AuxtraceError, // 72

    Unknown(u32)
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
            68 => EventType::FinishedRound,
            _ => EventType::Unknown(event_type)
        }
    }

    fn is_unknown(&self) -> bool {
        match *self {
            EventType::Unknown(_) => true,
            _ => false
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

#[derive(Debug)]
pub struct MMAP2Record {
    ptid: ThreadId,
    addr: u64,
    len: u64,
    pgoff: u64,
    maj: u32,
    min: u32,
    ino: u64,
    ino_generation: u64,
    prot: u32,
    flags: u32,
    filename: String,
    //TODO: sample_id: SampleId
}

named!(parse_mmap2_event<&[u8], EventData>,
    chain!(
        ptid: parse_thread_id ~
        addr: le_u64 ~
        len: le_u64 ~
        pgoff: le_u64 ~
        maj: le_u32 ~
        min: le_u32 ~
        ino: le_u64 ~
        ino_generation: le_u64 ~
        prot: le_u32 ~
        flags: le_u32 ~
        filename: parse_c_string,
        // TODO: sample_id: parse_sample_id,
        || EventData::MMAP2(MMAP2Record {
                ptid: ptid,
                addr: addr,
                len: len,
                pgoff: pgoff,
                maj: maj,
                min: min,
                ino: ino,
                ino_generation: ino_generation,
                prot: prot,
                flags: flags,
                filename: unsafe { String::from_utf8_unchecked(filename.to_vec()) }
        })
    )
);


/// We use the same read format for READ_FORMAT_GROUP and non-grouped reads for simplicity
#[derive(Default, Debug)]
pub struct ReadFormat {
    /// if PERF_FORMAT_TOTAL_TIME_ENABLED
    time_enabled: Option<u64>,
    /// if PERF_FORMAT_TOTAL_TIME_RUNNING
    time_running: Option<u64>,
    /// Collection of (value, Some(id) if PERF_FORMAT_ID)
    values: Vec<(u64, Option<u64>)>
}

fn parse_read_value(input: &[u8], flags: ReadFormatFlags) -> IResult<&[u8], (u64, Option<u64>)> {
    chain!(input,
        value: le_u64 ~
        id: cond!(flags.has_id(), le_u64),
        || (value, id)
    )
}

fn parse_read_format(input: &[u8], flags: ReadFormatFlags) -> IResult<&[u8], ReadFormat> {
    if flags.has_group() {
        return chain!(input,
            nr: le_u64 ~
            time_enabled: cond!(flags.has_total_time_enabled(), le_u64) ~
            time_running: cond!(flags.has_total_time_running(), le_u64) ~
            values: count!(call!(parse_read_value, flags), nr as usize),
            || ReadFormat {
                time_enabled: time_enabled,
                time_running: time_running,
                values: values
        });
    }
    else {
        return chain!(input,
            value: le_u64 ~
            time_enabled: cond!(flags.has_total_time_enabled(), le_u64) ~
            time_running: cond!(flags.has_total_time_running(), le_u64) ~
            id: cond!(flags.has_id(), le_u64),
            || ReadFormat {
                time_enabled: time_enabled,
                time_running: time_running,
                values: vec![(value, id)]
        });
    }
}

#[derive(Debug)]
pub struct ReadRecord {
    pid: u32,
    tid: u32,
    value: ReadFormat,
}

#[derive(Debug)]
struct BranchEntry {
    pub from: u64,
    pub to: u64,
    flags: u64,
}

named!(parse_branch_entry<&[u8], BranchEntry>,
    chain!(
        from: le_u64 ~
        to: le_u64 ~
        flags: le_u64,
        || BranchEntry {
            from: from,
            to: to,
            flags: flags,
        }
    )
);

fn parse_branch_entries(input: &[u8], flags: SampleFormatFlags) -> IResult<&[u8], Vec<BranchEntry>> {
    // TODO: bug? https://github.com/Geal/nom/issues/302
    assert!(flags.has_branch_stack() && flags.has_regs_user());
    chain!(input,
        // TODO: bug? https://github.com/Geal/nom/issues/302
        //bnr: cond!(flags.has_branch_stack(), le_u64) ~
        //entries: cond!(flags.has_branch_stack() && flags.has_regs_user(), count!(parse_branch_entry, 3)),
        bnr: le_u64 ~
        entries: count!(parse_branch_entry, bnr as usize),
        || entries
    )
}

/// This record indicates a sample.
#[derive(Debug)]
pub struct SampleRecord {
    /// if PERF_SAMPLE_IDENTIFIER
    sample_id: Option<u64>,
    /// if PERF_SAMPLE_IP
    ip: Option<u64>,
    /// if PERF_SAMPLE_TID
    ptid: Option<ThreadId>,
    /// if PERF_SAMPLE_TIME
    time: Option<u64>,
    /// if PERF_SAMPLE_ADDR

    addr: Option<u64>,
    /// if PERF_SAMPLE_ID
    id: Option<u64>,
    /// if PERF_SAMPLE_STREAM_ID
    stream_id: Option<u64>,
    /// if PERF_SAMPLE_CPU
    cpu: Option<Cpu>,
    /// if PERF_SAMPLE_PERIOD
    period: Option<u64>,
    /// if PERF_SAMPLE_READ
    v: Option<ReadFormat>,
    /// if PERF_SAMPLE_CALLCHAIN
    ips: Option<Vec<u64>>,
    /// if PERF_SAMPLE_RAW
    raw: Option<Vec<u8>>,
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
    abi: Option<u64>,
    /// if PERF_SAMPLE_REGS_INTR
    regs_intr: Option<Vec<u64>>
}

pub fn parse_sample_event<'a>(input: &'a [u8], attr: &'a EventAttr) -> IResult<&'a [u8], EventData> {
    let flags = attr.sample_type;
    let regcnt_user = attr.sample_regs_user.count_ones() as usize;
    let regcnt_intr = attr.sample_regs_intr.count_ones() as usize;
    chain!(input,
        sample_id: cond!(flags.has_identifier(), le_u64) ~
        ip: cond!(flags.has_ip(), le_u64) ~
        ptid: cond!(flags.has_tid(), parse_thread_id) ~
        time: cond!(flags.has_time(), le_u64) ~
        addr: cond!(flags.has_addr(), le_u64) ~
        id: cond!(flags.has_sample_id(), le_u64) ~
        stream_id: cond!(flags.has_stream_id(), le_u64) ~
        cpu: cond!(flags.has_cpu(), parse_cpu) ~
        period: cond!(flags.has_period(), le_u64) ~
        v: cond!(flags.has_read(), call!(parse_read_format, attr.read_format)) ~
        ips: cond!(flags.has_callchain(), parse_vec_u64) ~
        raw: cond!(flags.has_raw(), parse_vec_u32_u8) ~
        lbr: cond!(flags.has_branch_stack(), call!(parse_branch_entries, flags)) ~
        abi_user: cond!(flags.has_stack_user(), le_u64) ~
        regs_user: cond!(flags.has_stack_user(), call!(parse_vec_u64_variable, regcnt_user)) ~
        user_stack_len: cond!(flags.has_stack_user(), le_u64) ~
        user_stack: cond!(flags.has_stack_user(), call!(parse_vec_u8_variable, user_stack_len.unwrap() as usize)) ~
        dyn_size: cond!(flags.has_stack_user() && user_stack_len.unwrap() != 0, le_u64) ~
        weight: cond!(flags.has_weight(), le_u64) ~
        data_src: cond!(flags.has_data_src(), le_u64) ~
        transaction: cond!(flags.has_transaction(), le_u64) ~
        abi: cond!(flags.has_regs_intr(), le_u64) ~
        regs_intr: cond!(flags.has_regs_intr(), call!(parse_vec_u64_variable, regcnt_intr)),
        || EventData::Sample(SampleRecord {
                sample_id: sample_id,
                ip: ip,
                ptid: ptid,
                time: time,
                addr: addr,
                id: id,
                stream_id: stream_id,
                cpu: cpu,
                period: period,
                v: v,
                ips: ips,
                raw: raw,
                lbr: lbr,
                abi_user: abi_user,
                regs_user: regs_user,
                user_stack: user_stack,
                dyn_size: dyn_size,
                weight: weight,
                data_src: data_src,
                transaction: transaction,
                abi: abi,
                regs_intr: regs_intr
        })
    )
}

#[derive(Debug)]
pub struct CommRecord {
    ptid: ThreadId,
    comm: String,
    // TODO: sample_id
}

pub fn parse_comm_event(input: &[u8]) -> IResult<&[u8], EventData> {
    chain!(input,
        ptid: parse_thread_id ~
        comm: parse_c_string,
        // TODO: sample_id: parse_sample_id,
        || EventData::Comm(CommRecord {
                ptid: ptid,
                comm: unsafe { String::from_utf8_unchecked(comm.to_vec()) }
        })
    )
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
    Comm(CommRecord),
    Exit(ExitRecord),
    Throttle(ThrottleRecord),
    Unthrottle(UnthrottleRecord),
    Fork(ForkRecord),
    //Read(ReadRecord),
    Sample(SampleRecord),
    MMAP2(MMAP2Record),
    None,
}

fn is_nul_byte(c: &u8) -> bool {
    *c == 0x0
}

named!(parse_c_string, take_till!(is_nul_byte));

named!(parse_vec_u64<&[u8], Vec<u64> >,
    chain!(
        len: le_u64 ~
        vec: count!(le_u64, len as usize),
        || vec
    )
);

named!(parse_vec_u32_u8<&[u8], Vec<u8> >,
    chain!(
        len: le_u32 ~
        vec: count!(le_u8, len as usize),
        || vec
    )
);

fn parse_vec_u64_variable(input: &[u8], count: usize) -> IResult<&[u8], Vec<u64>> {
    count!(input, le_u64, count)
}

fn parse_vec_u8_variable(input: &[u8], count: usize) -> IResult<&[u8], Vec<u8>> {
    count!(input, le_u8, count)
}

fn no_event(input: &[u8]) -> IResult<&[u8], EventData> {
    IResult::Done(input, EventData::None)
}

/// Parse a file section
fn parse_event<'a>(input: &'a [u8], attrs: &'a Vec<EventAttr>) -> IResult<&'a [u8], Event> {
    chain!(input,
        header: parse_event_header ~
        event: alt!(
            cond_reduce!(header.event_type == EventType::Mmap, parse_mmap_event) |
            cond_reduce!(header.event_type == EventType::Mmap2, parse_mmap2_event) |
            cond_reduce!(header.event_type == EventType::Comm, parse_comm_event) |
            cond_reduce!(header.event_type == EventType::Exit, parse_exit_event) |
            cond_reduce!(header.event_type == EventType::Sample, call!(parse_sample_event, &attrs[0])) |
            cond_reduce!(header.event_type == EventType::Fork, parse_fork_event) |
            cond_reduce!(header.event_type == EventType::Unthrottle, parse_unthrottle_event) |
            cond_reduce!(header.event_type == EventType::Throttle, parse_throttle_event) |
            cond_reduce!(header.event_type == EventType::FinishedRound, no_event) |
            cond_reduce!(header.event_type.is_unknown(), no_event)
        ),
        || Event { header: header, data: event }
    )
}

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
            take_bits!(u8, 1) ~

            branch_stack: take_bits!(u8, 1) ~
            numa_topology: take_bits!(u8, 1) ~
            cpu_topology: take_bits!(u8, 1) ~
            event_desc: take_bits!(u8, 1) ~
            cmdline: take_bits!(u8, 1) ~
            total_mem: take_bits!(u8, 1) ~
            cpuid: take_bits!(u8, 1) ~
            cpudesc: take_bits!(u8, 1) ~

            take_bits!(u8, 6) ~ // padding
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
        take!(29), // reserved
        || PerfFileHeader { size: size, attr_size: attr_size, attrs: attrs, data: data, event_types: event_types, flags: flags }
    )
);

#[derive(Debug)]
pub struct EventAttr {
    pub attr_type: EventAttrType,
    pub size: u32,
    pub config: u64,
    pub sample_period_freq: u64,
    pub sample_type: SampleFormatFlags,
    pub read_format: ReadFormatFlags,
    pub settings: EventAttrFlags,

    pub wakeup_events_watermark: u32,
    pub bp_type: u32,

    pub config1_or_bp_addr: u64,
    pub config2_or_bp_len: u64,

    pub branch_sample_type: u64,
    pub sample_regs_user: u64,
    pub sample_stack_user: u32,
    pub clock_id: i32,
    pub sample_regs_intr: u64,
    pub aux_watermark: u32,
}

#[derive(Debug)]
pub enum EventAttrType {
    Hardware,
    Software,
    TracePoint,
    HwCache,
    Raw,
    Breakpoint
}

impl EventAttrType {
    fn new(attr_type: u32) -> EventAttrType {
        match attr_type {
            0 => EventAttrType::Hardware,
            1 => EventAttrType::Software,
            2 => EventAttrType::TracePoint,
            3 => EventAttrType::HwCache,
            4 => EventAttrType::Raw,
            5 => EventAttrType::Breakpoint,
            _ => panic!("Unknown Event Attribute type?")
        }
    }
}


bitflags!{
    #[derive(Debug)]
    flags ReadFormatFlags: u64 {
        /// Adds the 64-bit time_enabled field.  This can be used to calculate estimated totals if the PMU is overcommitted
        /// and multiplexing is happening.
        const FORMAT_TOTAL_TIME_ENABLED = 1,
        /// Adds the 64-bit time_running field.  This can be used to calculate estimated totals if the PMU is  overcommitted
        /// and  multiplexing is happening.
        const FORMAT_TOTAL_TIME_RUNNING = 2,
        /// Adds a 64-bit unique value that corresponds to the event group.
        const FORMAT_ID = 4,
        /// Allows all counter values in an event group to be read with one read.
        const FORMAT_GROUP = 8,
    }
}

impl ReadFormatFlags {
    pub fn has_total_time_enabled(&self) -> bool {
        self.contains(FORMAT_TOTAL_TIME_ENABLED)
    }

    pub fn has_total_time_running(&self) -> bool {
        self.contains(FORMAT_TOTAL_TIME_RUNNING)
    }

    pub fn has_id(&self) -> bool {
        self.contains(FORMAT_ID)
    }

    pub fn has_group(&self) -> bool {
        self.contains(FORMAT_GROUP)
    }
}

// Generated by using `cat /usr/include/linux/perf_event.h | grep PERF_SAMPLE_`
bitflags!{
    #[derive(Debug)]
    flags SampleFormatFlags: u64 {
        /// Records instruction pointer.
        const PERF_SAMPLE_IP = 1 << 0,
        /// Records the process and thread IDs.
        const PERF_SAMPLE_TID = 1 << 1,
        /// Records a timestamp.
        const PERF_SAMPLE_TIME = 1 << 2,
        /// Records an address, if applicable.
        const PERF_SAMPLE_ADDR = 1 << 3,
        /// Record counter values for all events in a group, not just the group leader.
        const PERF_SAMPLE_READ = 1 << 4,
        /// Records the callchain (stack backtrace).
        const PERF_SAMPLE_CALLCHAIN = 1 << 5,
        /// Records a unique ID for the opened event's group leader.
        const PERF_SAMPLE_ID = 1 << 6,
        /// Records CPU number.
        const PERF_SAMPLE_CPU = 1 << 7,
        /// Records the current sampling period.
        const PERF_SAMPLE_PERIOD = 1 << 8,
        /// Records  a  unique  ID  for  the  opened  event.  Unlike PERF_SAMPLE_ID the actual ID is returned, not the group
        /// leader.  This ID is the same as the one returned by PERF_FORMAT_ID.
        const PERF_SAMPLE_STREAM_ID = 1 << 9,
        /// Records additional data, if applicable.  Usually returned by tracepoint events.
        const PERF_SAMPLE_RAW = 1 << 10,
        /// This provides a record of recent branches, as provided by CPU branch  sampling  hardware  (such  as  Intel  Last
        /// Branch Record).  Not all hardware supports this feature.
        /// See the branch_sample_type field for how to filter which branches are reported.
        const PERF_SAMPLE_BRANCH_STACK = 1 << 11,
        /// Records the current user-level CPU register state (the values in the process before the kernel was called).
        const PERF_SAMPLE_REGS_USER = 1 << 12,
        /// Records the user level stack, allowing stack unwinding.
        const PERF_SAMPLE_STACK_USER = 1 << 13,
        /// Records a hardware provided weight value that expresses how costly the sampled event was.
        /// This allows the hardware to highlight expensive events in a profile.
        const PERF_SAMPLE_WEIGHT = 1 << 14,
        /// Records the data source: where in the memory hierarchy the data associated with the sampled instruction came from.
        /// This is only available if the underlying hardware supports this feature.
        const PERF_SAMPLE_DATA_SRC = 1 << 15,
        const PERF_SAMPLE_IDENTIFIER = 1 << 16,
        const PERF_SAMPLE_TRANSACTION = 1 << 17,
        const PERF_SAMPLE_REGS_INTR = 1 << 18,
    }
}

impl SampleFormatFlags {
    pub fn has_ip(&self) -> bool {
        self.contains(PERF_SAMPLE_IP)
    }

    pub fn has_tid(&self) -> bool {
        self.contains(PERF_SAMPLE_TID)
    }

    pub fn has_time(&self) -> bool {
        self.contains(PERF_SAMPLE_TIME)
    }

    pub fn has_addr(&self) -> bool {
        self.contains(PERF_SAMPLE_ADDR)
    }

    pub fn has_read(&self) -> bool {
        self.contains(PERF_SAMPLE_READ)
    }

    pub fn has_callchain(&self) -> bool {
        self.contains(PERF_SAMPLE_CALLCHAIN)
    }

    pub fn has_sample_id(&self) -> bool {
        self.contains(PERF_SAMPLE_ID)
    }

    pub fn has_cpu(&self) -> bool {
        self.contains(PERF_SAMPLE_CPU)
    }

    pub fn has_period(&self) -> bool {
        self.contains(PERF_SAMPLE_PERIOD)
    }

    pub fn has_stream_id(&self) -> bool {
        self.contains(PERF_SAMPLE_STREAM_ID)
    }

    pub fn has_raw(&self) -> bool {
        self.contains(PERF_SAMPLE_RAW)
    }

    pub fn has_branch_stack(&self) -> bool {
        self.contains(PERF_SAMPLE_BRANCH_STACK)
    }

    pub fn has_regs_user(&self) -> bool {
        self.contains(PERF_SAMPLE_REGS_USER)
    }

    pub fn has_stack_user(&self) -> bool {
        self.contains(PERF_SAMPLE_STACK_USER)
    }

    pub fn has_weight(&self) -> bool {
        self.contains(PERF_SAMPLE_WEIGHT)
    }

    pub fn has_data_src(&self) -> bool {
        self.contains(PERF_SAMPLE_DATA_SRC)
    }

    pub fn has_identifier(&self) -> bool {
        self.contains(PERF_SAMPLE_IDENTIFIER)
    }

    pub fn has_transaction(&self) -> bool {
        self.contains(PERF_SAMPLE_TRANSACTION)
    }

    pub fn has_regs_intr(&self) -> bool {
        self.contains(PERF_SAMPLE_REGS_INTR)
    }
}

bitflags! {
    #[derive(Debug)]
    flags EventAttrFlags: u64 {
        /// off by default
        const EVENT_ATTR_DISABLED       =  1 << 0,
        /// children inherit it
        const EVENT_ATTR_INHERIT        =  1 << 1,
        /// must always be on PMU
        const EVENT_ATTR_PINNED         =  1 << 2,
        /// only group on PMU
        const EVENT_ATTR_EXCLUSIVE      =  1 << 3,
        /// don't count user
        const EVENT_ATTR_EXCLUDE_USER   =  1 << 4,
        /// ditto kernel
        const EVENT_ATTR_EXCLUDE_KERNEL =  1 << 5,
        /// ditto hypervisor
        const EVENT_ATTR_EXCLUDE_HV     =  1 << 6,
        /// don't count when idle
        const EVENT_ATTR_EXCLUDE_IDLE   =  1 << 7,
        /// include mmap data
        const EVENT_ATTR_MMAP           =  1 << 8,
        /// include comm data
        const EVENT_ATTR_COMM           =  1 << 9,
        /// use freq, not period
        const EVENT_ATTR_FREQ           =  1 << 10,
        /// per task counts
        const EVENT_ATTR_INHERIT_STAT   =  1 << 11,
        /// next exec enables
        const EVENT_ATTR_ENABLE_ON_EXEC =  1 << 12,
        /// trace fork/exit
        const EVENT_ATTR_TASK           =  1 << 13,
        /// wakeup_watermark
        const EVENT_ATTR_WATERMARK      =  1 << 14,

        /// SAMPLE_IP can have arbitrary skid
        const EVENT_ATTR_SAMPLE_IP_ARBITRARY_SKID = 0 << 15,
        /// SAMPLE_IP must have constant skid
        const EVENT_ATTR_SAMPLE_IP_CONSTANT_SKID = 1 << 15,
        /// SAMPLE_IP requested to have 0 skid
        const EVENT_ATTR_SAMPLE_IP_REQ_ZERO_SKID = 2 << 15,
        /// SAMPLE_IP must have 0 skid
        const EVENT_ATTR_SAMPLE_IP_ZERO_SKID = 3 << 15,

        /// non-exec mmap data
        const EVENT_ATTR_MMAP_DATA =  1 << 17,
        /// sample_type all events
        const EVENT_ATTR_SAMPLE_ID_ALL =  1 << 18,
        /// don't count in host
        const EVENT_ATTR_EXCLUDE_HOST =  1 << 19,
        /// don't count in guest
        const EVENT_ATTR_EXCLUDE_GUEST =  1 << 20,
        /// exclude kernel callchains
        const EVENT_ATTR_EXCLUDE_CALLCHAIN_KERNEL = 1 << 21,
        /// exclude user callchains
        const EVENT_ATTR_EXCLUDE_CALLCHAIN_USER = 1 << 22,
        /// include mmap with inode data
        const EVENT_ATTR_MMAP2  =  1 << 23,
    }
}

/// Parse a perf header
named!(parse_event_attr<&[u8], EventAttr>,
    chain!(
        attr_type: le_u32 ~
        size: le_u32 ~
        config: le_u64 ~
        sample_period_freq: le_u64 ~
        sample_type: le_u64 ~
        read_format: le_u64 ~
        settings: le_u64 ~
        wakeup_events_watermark: le_u32 ~
        bp_type: le_u32 ~
        config1_or_bp_addr: le_u64 ~
        config2_or_bp_len: le_u64 ~
        branch_sample_type: le_u64 ~
        sample_regs_user: le_u64 ~
        sample_stack_user: le_u32 ~
        clock_id: le_i32 ~
        sample_regs_intr: le_u64 ~
        aux_watermark: le_u32 ~
        le_u32, // reserved
        || EventAttr {
            attr_type: EventAttrType::new(attr_type),
            size: size,
            config: config,
            sample_period_freq: sample_period_freq,
            sample_type: SampleFormatFlags::from_bits_truncate(sample_type),
            read_format: ReadFormatFlags::from_bits_truncate(read_format),
            settings: EventAttrFlags::from_bits_truncate(settings),
            wakeup_events_watermark: wakeup_events_watermark,
            bp_type: bp_type,
            config1_or_bp_addr: config1_or_bp_addr,
            config2_or_bp_len: config2_or_bp_len,
            branch_sample_type: branch_sample_type,
            sample_regs_user: sample_regs_user,
            sample_stack_user: sample_stack_user,
            clock_id: clock_id,
            sample_regs_intr: sample_regs_intr,
            aux_watermark: aux_watermark,
        }
));


#[derive(Debug)]
pub struct PerfFile {
    bytes: Vec<u8>,
    pub header: PerfFileHeader,
    pub attrs: Vec<EventAttr>
    //sections: Vec<PerfFileSection>,
}

impl PerfFile {


    pub fn new(bytes: Vec<u8>) -> PerfFile {
        let header = match parse_header(bytes.as_slice()) {
            IResult::Done(_, h) => h,
            IResult::Error(e) => panic!("{:?}", e),
            IResult::Incomplete(_) => panic!("Incomplete data?"),
        };
        let attrs = {
            let attr_size = header.attr_size as usize;
            let slice: &[u8] = &bytes[header.attrs.start()..header.attrs.end()];
            slice.chunks(attr_size).map(|c| parse_event_attr(c).unwrap().1 ).collect()
        };

        PerfFile { bytes: bytes, header: header, attrs: attrs }
    }

    pub fn data(&self) {
        let mut slice: &[u8] = &self.bytes[self.header.data.start()..self.header.data.end()];

        while slice.len() > 8 {
            let r = parse_event(slice, &self.attrs);
            match r {
                IResult::Done(rest, ev) => {
                    println!("{:?}", ev);
                    println!("Parsed bytes: {:?}", slice.len() - rest.len());
                    println!("Event size: {:?}", ev.header.size());
                    println!("Padding: {:?}", ev.header.size() - (slice.len() - rest.len()) );
                    slice = rest.split_at( ev.header.size() - (slice.len() - rest.len()) ).1;
                    println!("Remaining slice: {:?}", slice.len());
                },
                IResult::Error(_) => { panic!("Got error"); },
                IResult::Incomplete(_) => { panic!("Got incomplete:"); }
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
