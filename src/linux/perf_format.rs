//! Contains the various data format structures as used by perf
//!
//! In order to parse these structures from perf files or perf MMAP buffers, please
//! have a look at the functions in parser.rs.

use bitflags::*;

/// Unique thread descriptor. Used in many different perf structures.
#[derive(Debug)]
pub struct ThreadId {
    pub pid: i32,
    pub tid: i32,
}

/// Generic CPU description. Used in many different perf structures.
#[derive(Debug)]
pub struct Cpu {
    pub cpu: u32,
    pub res: u32,
}

#[derive(Debug)]
pub struct SampleId {
    /// if PERF_SAMPLE_TID set
    pub ptid: ThreadId,
    /// if PERF_SAMPLE_TIME set
    pub time: u64,
    /// if PERF_SAMPLE_ID set
    pub id: u64,
    /// if PERF_SAMPLE_STREAM_ID set
    pub stream_id: u64,
    /// if PERF_SAMPLE_CPU set
    pub cpu: Cpu,
    /// if PERF_SAMPLE_IDENTIFIER set
    pub identifier: u64,
}

#[derive(Debug)]
pub struct Event {
    pub header: EventHeader,
    pub data: EventData,
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
    BuildId(BuildIdRecord),
    None,
}

#[derive(Debug)]
pub struct EventHeader {
    pub event_type: EventType,
    pub misc: u16,
    pub size: u16,
}

impl EventHeader {
    pub fn size(&self) -> usize {
        self.size as usize
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum EventType {
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
    BuildId,       // 67
    FinishedRound, // 68
    // RecordIdIndex, // 69
    // AuxTraceInfo, // 70
    // AuxTrace, // 71
    // AuxtraceError, // 72
    Unknown(u32),
}

impl EventType {
    pub fn new(event_type: u32) -> EventType {
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
            67 => EventType::BuildId,
            68 => EventType::FinishedRound,
            _ => EventType::Unknown(event_type),
        }
    }

    pub fn is_unknown(&self) -> bool {
        match *self {
            EventType::Unknown(_) => true,
            _ => false,
        }
    }
}

/// This record indicates a fork event.
#[derive(Debug)]
pub struct ForkRecord {
    pub pid: u32,
    pub ppid: u32,
    pub tid: u32,
    pub ptid: u32,
    pub time: u64,
    // TOOD: sample_id
}

/// This record indicates a process exit event.
#[derive(Debug)]
pub struct ExitRecord {
    pub pid: u32,
    pub ppid: u32,
    pub tid: u32,
    pub ptid: u32,
    pub time: u64, // TOOD: sample_id
}

#[derive(Debug)]
pub struct ThrottleRecord {
    pub time: u64,
    pub id: u64,
    pub stream_id: u64, // TODO: sample id?
}

#[derive(Debug)]
pub struct UnthrottleRecord {
    pub time: u64,
    pub id: u64,
    pub stream_id: u64, // TODO: sample id?
}

/// The MMAP events record the PROT_EXEC mappings so that we can correlate user-space IPs to code.
#[derive(Debug)]
pub struct MMAPRecord {
    pub pid: i32,
    pub tid: u32,
    pub addr: u64,
    pub len: u64,
    pub pgoff: u64,
    pub filename: String,
}

#[derive(Debug)]
pub struct MMAP2Record {
    pub ptid: ThreadId,
    pub addr: u64,
    pub len: u64,
    pub pgoff: u64,
    pub maj: u32,
    pub min: u32,
    pub ino: u64,
    pub ino_generation: u64,
    pub prot: u32,
    pub flags: u32,
    pub filename: String,
    //TODO: sample_id: SampleId
}

/// We use the same read format for READ_FORMAT_GROUP and non-grouped reads for simplicity
#[derive(Default, Debug)]
pub struct ReadFormat {
    /// if PERF_FORMAT_TOTAL_TIME_ENABLED
    pub time_enabled: Option<u64>,
    /// if PERF_FORMAT_TOTAL_TIME_RUNNING
    pub time_running: Option<u64>,
    /// Collection of (value, Some(id) if PERF_FORMAT_ID)
    pub values: Vec<(u64, Option<u64>)>,
}

#[derive(Debug)]
pub struct ReadRecord {
    pub pid: u32,
    pub tid: u32,
    pub value: ReadFormat,
}

#[derive(Debug)]
pub struct BranchEntry {
    pub from: u64,
    pub to: u64,
    pub flags: u64,
}

/// This record indicates a sample.
#[derive(Debug)]
pub struct SampleRecord {
    /// if PERF_SAMPLE_IDENTIFIER
    pub sample_id: Option<u64>,
    /// if PERF_SAMPLE_IP
    pub ip: Option<u64>,
    /// if PERF_SAMPLE_TID
    pub ptid: Option<ThreadId>,
    /// if PERF_SAMPLE_TIME
    pub time: Option<u64>,
    /// if PERF_SAMPLE_ADDR
    pub addr: Option<u64>,
    /// if PERF_SAMPLE_ID
    pub id: Option<u64>,
    /// if PERF_SAMPLE_STREAM_ID
    pub stream_id: Option<u64>,
    /// if PERF_SAMPLE_CPU
    pub cpu: Option<Cpu>,
    /// if PERF_SAMPLE_PERIOD
    pub period: Option<u64>,
    /// if PERF_SAMPLE_READ
    pub v: Option<ReadFormat>,
    /// if PERF_SAMPLE_CALLCHAIN
    pub ips: Option<Vec<u64>>,
    /// if PERF_SAMPLE_RAW
    pub raw: Option<Vec<u8>>,
    /// if PERF_SAMPLE_REGS_USER & PERF_SAMPLE_BRANCH_STACK
    pub lbr: Option<Vec<BranchEntry>>,
    /// PERF_SAMPLE_STACK_USER
    pub abi_user: Option<u64>,
    /// PERF_SAMPLE_STACK_USER
    pub regs_user: Option<Vec<u64>>,
    /// PERF_SAMPLE_STACK_USER
    pub user_stack: Option<Vec<u8>>,
    /// PERF_SAMPLE_STACK_USER
    pub dyn_size: Option<u64>,
    /// if PERF_SAMPLE_WEIGHT
    pub weight: Option<u64>,
    /// if PERF_SAMPLE_DATA_SRC
    pub data_src: Option<u64>,
    /// if PERF_SAMPLE_TRANSACTION
    pub transaction: Option<u64>,
    /// if PERF_SAMPLE_REGS_INTR
    pub abi: Option<u64>,
    /// if PERF_SAMPLE_REGS_INTR
    pub regs_intr: Option<Vec<u64>>,
}

#[derive(Debug)]
pub struct CommRecord {
    pub ptid: ThreadId,
    pub comm: String,
    // TODO: sample_id
}

#[derive(Debug)]
pub struct LostRecord {}

#[derive(Debug)]
pub struct BuildIdRecord {
    pub pid: i32,
    pub build_id: Vec<u8>,
    pub filename: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
    PmuMappings,
}

#[derive(Debug)]
pub struct HeaderFlags {
    pub nrcpus: bool,
    pub arch: bool,
    pub version: bool,
    pub osrelease: bool,
    pub hostname: bool,
    pub build_id: bool,
    pub tracing_data: bool,
    pub branch_stack: bool,
    pub numa_topology: bool,
    pub cpu_topology: bool,
    pub event_desc: bool,
    pub cmdline: bool,
    pub total_mem: bool,
    pub cpuid: bool,
    pub cpudesc: bool,
    pub group_desc: bool,
    pub pmu_mappings: bool,
}

impl HeaderFlags {
    pub fn collect(&self) -> Vec<HeaderFlag> {
        // The order in which these flags are pushed is important!
        // Must be in the exact order as they appear in the binary format
        // otherwise we parse the wrong file sections!
        let mut flags = Vec::with_capacity(17);

        if self.tracing_data {
            flags.push(HeaderFlag::TracingData);
        }
        if self.build_id {
            flags.push(HeaderFlag::BuildId);
        }
        if self.hostname {
            flags.push(HeaderFlag::Hostname);
        }
        if self.osrelease {
            flags.push(HeaderFlag::OsRelease);
        }
        if self.version {
            flags.push(HeaderFlag::Version);
        }
        if self.arch {
            flags.push(HeaderFlag::Arch);
        }
        if self.nrcpus {
            flags.push(HeaderFlag::NrCpus);
        }

        if self.cpudesc {
            flags.push(HeaderFlag::CpuDesc);
        }
        if self.cpuid {
            flags.push(HeaderFlag::CpuId);
        }
        if self.total_mem {
            flags.push(HeaderFlag::TotalMem);
        }
        if self.cmdline {
            flags.push(HeaderFlag::CmdLine);
        }
        if self.event_desc {
            flags.push(HeaderFlag::EventDesc);
        }
        if self.cpu_topology {
            flags.push(HeaderFlag::CpuTopology);
        }
        if self.numa_topology {
            flags.push(HeaderFlag::NumaTopology);
        }
        if self.branch_stack {
            flags.push(HeaderFlag::BranchStack);
        }

        if self.pmu_mappings {
            flags.push(HeaderFlag::PmuMappings);
        }
        if self.group_desc {
            flags.push(HeaderFlag::GroupDesc);
        }
        flags
    }
}

#[derive(Debug)]
pub struct PerfFileHeader {
    pub size: u64,
    pub attr_size: u64,
    pub attrs: PerfFileSection,
    pub data: PerfFileSection,
    pub event_types: PerfFileSection,
    pub flags: HeaderFlags,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EventAttr {
    pub attr_type: u32,
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
    pub reserved: u32,
}

impl EventAttr {
    pub fn attr_type(&self) -> EventAttrType {
        EventAttrType::new(self.attr_type)
    }
}

impl Default for EventAttr {
    fn default() -> EventAttr {
        use std::mem;
        unsafe { mem::zeroed::<EventAttr>() }
    }
}

#[derive(Debug)]
pub enum EventAttrType {
    Hardware,
    Software,
    TracePoint,
    HwCache,
    Raw,
    Breakpoint,
    Unknown(u32),
}

impl EventAttrType {
    pub fn new(attr_type: u32) -> EventAttrType {
        match attr_type {
            0 => EventAttrType::Hardware,
            1 => EventAttrType::Software,
            2 => EventAttrType::TracePoint,
            3 => EventAttrType::HwCache,
            4 => EventAttrType::Raw,
            5 => EventAttrType::Breakpoint,
            _ => EventAttrType::Unknown(attr_type),
        }
    }
}

bitflags! {
    pub struct ReadFormatFlags: u64 {
        /// Adds the 64-bit time_enabled field.  This can be used to calculate estimated totals if the PMU is overcommitted
        /// and multiplexing is happening.
        const FORMAT_TOTAL_TIME_ENABLED = 1 << 0;
        /// Adds the 64-bit time_running field.  This can be used to calculate estimated totals if the PMU is  overcommitted
        /// and  multiplexing is happening.
        const FORMAT_TOTAL_TIME_RUNNING = 1 << 1;
        /// Adds a 64-bit unique value that corresponds to the event group.
        const FORMAT_ID = 1 << 2;
        /// Allows all counter values in an event group to be read with one read.
        const FORMAT_GROUP = 1 << 3;
    }
}

impl ReadFormatFlags {
    pub fn has_total_time_enabled(&self) -> bool {
        self.contains(ReadFormatFlags::FORMAT_TOTAL_TIME_ENABLED)
    }

    pub fn has_total_time_running(&self) -> bool {
        self.contains(ReadFormatFlags::FORMAT_TOTAL_TIME_RUNNING)
    }

    pub fn has_id(&self) -> bool {
        self.contains(ReadFormatFlags::FORMAT_ID)
    }

    pub fn has_group(&self) -> bool {
        self.contains(ReadFormatFlags::FORMAT_GROUP)
    }
}

// Generated by using `cat /usr/include/linux/perf_event.h | grep PERF_SAMPLE_`
bitflags! {
    pub struct SampleFormatFlags: u64 {
        /// Records instruction pointer.
        const PERF_SAMPLE_IP = 1 << 0;
        /// Records the process and thread IDs.
        const PERF_SAMPLE_TID = 1 << 1;
        /// Records a timestamp.
        const PERF_SAMPLE_TIME = 1 << 2;
        /// Records an address, if applicable.
        const PERF_SAMPLE_ADDR = 1 << 3;
        /// Record counter values for all events in a group, not just the group leader.
        const PERF_SAMPLE_READ = 1 << 4;
        /// Records the callchain (stack backtrace).
        const PERF_SAMPLE_CALLCHAIN = 1 << 5;
        /// Records a unique ID for the opened event's group leader.
        const PERF_SAMPLE_ID = 1 << 6;
        /// Records CPU number.
        const PERF_SAMPLE_CPU = 1 << 7;
        /// Records the current sampling period.
        const PERF_SAMPLE_PERIOD = 1 << 8;
        /// Records  a  unique  ID  for  the  opened  event.  Unlike PERF_SAMPLE_ID the actual ID is returned, not the group
        /// leader.  This ID is the same as the one returned by PERF_FORMAT_ID.
        const PERF_SAMPLE_STREAM_ID = 1 << 9;
        /// Records additional data, if applicable.  Usually returned by tracepoint events.
        const PERF_SAMPLE_RAW = 1 << 10;
        /// This provides a record of recent branches, as provided by CPU branch  sampling  hardware  (such  as  Intel  Last
        /// Branch Record).  Not all hardware supports this feature.
        /// See the branch_sample_type field for how to filter which branches are reported.
        const PERF_SAMPLE_BRANCH_STACK = 1 << 11;
        /// Records the current user-level CPU register state (the values in the process before the kernel was called).
        const PERF_SAMPLE_REGS_USER = 1 << 12;
        /// Records the user level stack, allowing stack unwinding.
        const PERF_SAMPLE_STACK_USER = 1 << 13;
        /// Records a hardware provided weight value that expresses how costly the sampled event was.
        /// This allows the hardware to highlight expensive events in a profile.
        const PERF_SAMPLE_WEIGHT = 1 << 14;
        /// Records the data source: where in the memory hierarchy the data associated with the sampled instruction came from.
        /// This is only available if the underlying hardware supports this feature.
        const PERF_SAMPLE_DATA_SRC = 1 << 15;
        const PERF_SAMPLE_IDENTIFIER = 1 << 16;
        const PERF_SAMPLE_TRANSACTION = 1 << 17;
        const PERF_SAMPLE_REGS_INTR = 1 << 18;
    }
}

impl SampleFormatFlags {
    pub fn has_ip(&self) -> bool {
        self.contains(SampleFormatFlags::PERF_SAMPLE_IP)
    }

    pub fn has_tid(&self) -> bool {
        self.contains(SampleFormatFlags::PERF_SAMPLE_TID)
    }

    pub fn has_time(&self) -> bool {
        self.contains(SampleFormatFlags::PERF_SAMPLE_TIME)
    }

    pub fn has_addr(&self) -> bool {
        self.contains(SampleFormatFlags::PERF_SAMPLE_ADDR)
    }

    pub fn has_read(&self) -> bool {
        self.contains(SampleFormatFlags::PERF_SAMPLE_READ)
    }

    pub fn has_callchain(&self) -> bool {
        self.contains(SampleFormatFlags::PERF_SAMPLE_CALLCHAIN)
    }

    pub fn has_sample_id(&self) -> bool {
        self.contains(SampleFormatFlags::PERF_SAMPLE_ID)
    }

    pub fn has_cpu(&self) -> bool {
        self.contains(SampleFormatFlags::PERF_SAMPLE_CPU)
    }

    pub fn has_period(&self) -> bool {
        self.contains(SampleFormatFlags::PERF_SAMPLE_PERIOD)
    }

    pub fn has_stream_id(&self) -> bool {
        self.contains(SampleFormatFlags::PERF_SAMPLE_STREAM_ID)
    }

    pub fn has_raw(&self) -> bool {
        self.contains(SampleFormatFlags::PERF_SAMPLE_RAW)
    }

    pub fn has_branch_stack(&self) -> bool {
        self.contains(SampleFormatFlags::PERF_SAMPLE_BRANCH_STACK)
    }

    pub fn has_regs_user(&self) -> bool {
        self.contains(SampleFormatFlags::PERF_SAMPLE_REGS_USER)
    }

    pub fn has_stack_user(&self) -> bool {
        self.contains(SampleFormatFlags::PERF_SAMPLE_STACK_USER)
    }

    pub fn has_weight(&self) -> bool {
        self.contains(SampleFormatFlags::PERF_SAMPLE_WEIGHT)
    }

    pub fn has_data_src(&self) -> bool {
        self.contains(SampleFormatFlags::PERF_SAMPLE_DATA_SRC)
    }

    pub fn has_identifier(&self) -> bool {
        self.contains(SampleFormatFlags::PERF_SAMPLE_IDENTIFIER)
    }

    pub fn has_transaction(&self) -> bool {
        self.contains(SampleFormatFlags::PERF_SAMPLE_TRANSACTION)
    }

    pub fn has_regs_intr(&self) -> bool {
        self.contains(SampleFormatFlags::PERF_SAMPLE_REGS_INTR)
    }
}

bitflags! {
    pub struct EventAttrFlags: u64 {
        /// off by default
        const EVENT_ATTR_DISABLED       =  1 << 0;
        /// children inherit it
        const EVENT_ATTR_INHERIT        =  1 << 1;
        /// must always be on PMU
        const EVENT_ATTR_PINNED         =  1 << 2;
        /// only group on PMU
        const EVENT_ATTR_EXCLUSIVE      =  1 << 3;
        /// don't count user
        const EVENT_ATTR_EXCLUDE_USER   =  1 << 4;
        /// ditto kernel
        const EVENT_ATTR_EXCLUDE_KERNEL =  1 << 5;
        /// ditto hypervisor
        const EVENT_ATTR_EXCLUDE_HV     =  1 << 6;
        /// don't count when idle
        const EVENT_ATTR_EXCLUDE_IDLE   =  1 << 7;
        /// include mmap data
        const EVENT_ATTR_MMAP           =  1 << 8;
        /// include comm data
        const EVENT_ATTR_COMM           =  1 << 9;
        /// use freq, not period
        const EVENT_ATTR_FREQ           =  1 << 10;
        /// per task counts
        const EVENT_ATTR_INHERIT_STAT   =  1 << 11;
        /// next exec enables
        const EVENT_ATTR_ENABLE_ON_EXEC =  1 << 12;
        /// trace fork/exit
        const EVENT_ATTR_TASK           =  1 << 13;
        /// wakeup_watermark
        const EVENT_ATTR_WATERMARK      =  1 << 14;

        /// SAMPLE_IP can have arbitrary skid
        const EVENT_ATTR_SAMPLE_IP_ARBITRARY_SKID = 0 << 15;
        /// SAMPLE_IP must have constant skid
        const EVENT_ATTR_SAMPLE_IP_CONSTANT_SKID = 1 << 15;
        /// SAMPLE_IP requested to have 0 skid
        const EVENT_ATTR_SAMPLE_IP_REQ_ZERO_SKID = 2 << 15;
        /// SAMPLE_IP must have 0 skid
        const EVENT_ATTR_SAMPLE_IP_ZERO_SKID = 3 << 15;

        /// non-exec mmap data
        const EVENT_ATTR_MMAP_DATA =  1 << 17;
        /// sample_type all events
        const EVENT_ATTR_SAMPLE_ID_ALL =  1 << 18;
        /// don't count in host
        const EVENT_ATTR_EXCLUDE_HOST =  1 << 19;
        /// don't count in guest
        const EVENT_ATTR_EXCLUDE_GUEST =  1 << 20;
        /// exclude kernel callchains
        const EVENT_ATTR_EXCLUDE_CALLCHAIN_KERNEL = 1 << 21;
        /// exclude user callchains
        const EVENT_ATTR_EXCLUDE_CALLCHAIN_USER = 1 << 22;
        /// include mmap with inode data
        const EVENT_ATTR_MMAP2  =  1 << 23;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PerfFileSection {
    pub offset: u64,
    pub size: u64,
}

impl PerfFileSection {
    pub fn start(&self) -> usize {
        self.offset as usize
    }

    pub fn end(&self) -> usize {
        (self.offset + self.size) as usize
    }
}

#[derive(Debug)]
pub struct NrCpus {
    /// How many CPUs are online
    pub online: u32,
    /// CPUs not yet online
    pub available: u32,
}

#[derive(Debug)]
pub struct EventDesc {
    pub attr: EventAttr,
    pub event_string: String,
    pub ids: Vec<u64>,
}

#[derive(Debug)]
pub struct CpuTopology {
    pub cores: Vec<String>,
    pub threads: Vec<String>,
}

#[derive(Debug)]
pub struct NumaNode {
    pub node_nr: u32,
    pub mem_total: u64,
    pub mem_free: u64,
    pub cpus: String,
}

#[derive(Debug)]
pub struct PmuMapping {
    pub pmu_type: u32,
    pub pmu_name: String,
}

#[derive(Debug)]
pub struct GroupDesc {
    pub string: String,
    pub leader_idx: u32,
    pub nr_members: u32,
}
