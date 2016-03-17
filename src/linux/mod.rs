//! A wrapper around perf_event open (http://lxr.free-electrons.com/source/tools/perf/design.txt)

use std::slice;
use std::fs::File;
use std::os::unix::io::FromRawFd;
use std::io;
use std::io::{Read, Error};
use std::mem;
use std::fmt;
use std::str;

use libc::{pid_t, MAP_SHARED, strlen};
use mmap;

#[allow(dead_code, non_camel_case_types)]
mod hw_breakpoint;
#[allow(dead_code, non_camel_case_types)]
mod perf_event;

use ::AbstractPerfCounter;
use x86::perfcnt::intel::description::{IntelPerformanceCounterDescription, Tuple};

const IOCTL: usize = 16;
const PERF_EVENT_OPEN: usize = 298;

fn perf_event_open(hw_event: perf_event::perf_event_attr,
                       pid: perf_event::__kernel_pid_t,
                       cpu:  ::libc::c_int,
                       group_fd:  ::libc::c_int,
                       flags:  ::libc::c_int) -> isize {
    unsafe {
        syscall!(PERF_EVENT_OPEN, &hw_event as *const perf_event::perf_event_attr as usize, pid, cpu, group_fd, flags) as isize
    }
}

fn ioctl(fd: ::libc::c_int, request: u64, value: ::libc::c_int) -> isize {
    unsafe {
        syscall!(IOCTL, fd, request, value) as isize
    }
}

pub struct PerfCounterBuilderLinux {
    group: isize,
    pid: pid_t,
    cpu: isize,
    flags: usize,
    attrs: perf_event::perf_event_attr,
}

impl Default for PerfCounterBuilderLinux {
    fn default () -> PerfCounterBuilderLinux {
        PerfCounterBuilderLinux {
            group: -1,
            pid: 0,
            cpu: -1,
            flags: 0,
            attrs: Default::default()
        }
    }
}

pub enum HardwareEventType {
    /// Total cycles.  Be wary of what happens during CPU frequency scaling.
    CPUCycles = perf_event::PERF_COUNT_HW_CPU_CYCLES as isize,

    /// Retired instructions.  Be careful, these can be affected by various issues, most notably hardware interrupt counts.
    Instructions = perf_event::PERF_COUNT_HW_INSTRUCTIONS as isize,

    /// Cache  accesses.  Usually this indicates Last Level Cache accesses but this may vary depending on your CPU. This may include prefetches and
    CacheReferences = perf_event::PERF_COUNT_HW_CACHE_REFERENCES as isize,

    /// Cache misses.  Usually this indicates Last Level Cache misses; this is intended to be used  in  conjunction with the
    CacheMisses = perf_event::PERF_COUNT_HW_CACHE_MISSES as isize,

    /// Retired branch instructions.  Prior to Linux 2.6.34, this used the wrong event on AMD processors.
    BranchInstructions = perf_event::PERF_COUNT_HW_BRANCH_INSTRUCTIONS as isize,

    /// Mispredicted branch instructions.
    BranchMisses = perf_event::PERF_COUNT_HW_BRANCH_MISSES as isize,

    /// Bus cycles, which can be different from total cycles.
    BusCycles = perf_event::PERF_COUNT_HW_BUS_CYCLES as isize,

    /// Stalled cycles during issue. (Since Linux 3.0)
    StalledCyclesFrontend = perf_event::PERF_COUNT_HW_STALLED_CYCLES_FRONTEND as isize,

    /// Stalled cycles during retirement. (Since Linux 3.0)
    StalledCyclesBackend = perf_event::PERF_COUNT_HW_STALLED_CYCLES_BACKEND as isize,

    /// Total cycles; not affected by CPU frequency scaling. (Since Linux 3.3)
    RefCPUCycles = perf_event::PERF_COUNT_HW_REF_CPU_CYCLES as isize,
}

pub enum SoftwareEventType {

    /// This reports the CPU clock, a high-resolution per-CPU timer.
    CpuClock = perf_event::PERF_COUNT_SW_CPU_CLOCK as isize,

    /// This reports a clock count specific to the task that is running.
    TaskClock = perf_event::PERF_COUNT_SW_TASK_CLOCK as isize,

    /// This reports the number of page faults.
    PageFaults = perf_event::PERF_COUNT_SW_PAGE_FAULTS as isize,

    /// This counts context switches.
    ///
    /// Until Linux 2.6.34, these were all reported as user-space events, after that
    /// they are reported as happening in the kernel.
    ContextSwitches = perf_event::PERF_COUNT_SW_CONTEXT_SWITCHES as isize,

    /// This reports the number of times the process has migrated to a new CPU.
    CpuMigrations = perf_event::PERF_COUNT_SW_CPU_MIGRATIONS as isize,

    /// This counts the number of minor page faults.  These did not require disk I/O to handle.
    PageFaultsMin = perf_event::PERF_COUNT_SW_PAGE_FAULTS_MIN as isize,

    /// This counts the number of major page faults.  These required disk I/O to handle.
    PageFaultsMaj = perf_event::PERF_COUNT_SW_PAGE_FAULTS_MAJ as isize,

    /// This counts the number of alignment faults.
    ///
    /// These happen when unaligned memory accesses happen; the kernel
    /// can handle these but it reduces performance. This happens only on some architectures (never on x86).
    ///
    /// (Since Linux 2.6.33)
    AlignmentFaults = perf_event::PERF_COUNT_SW_ALIGNMENT_FAULTS as isize,

    /// This counts the number of emulation faults.  The kernel sometimes traps on unimplemented  instructions  and
    /// emulates them for user space.  This can negatively impact performance.
    ///
    /// (Since Linux 2.6.33)
    EmulationFaults = perf_event::PERF_COUNT_SW_EMULATION_FAULTS as isize,

}

pub enum CacheId {
    /// For measuring Level 1 Data Cache
    L1D = perf_event::PERF_COUNT_HW_CACHE_L1D as isize,

    /// For measuring Level 1 Instruction Cache
    L1I = perf_event::PERF_COUNT_HW_CACHE_L1I as isize,

    /// For measuring Last-Level Cache
    LL = perf_event::PERF_COUNT_HW_CACHE_LL as isize,

    /// For measuring the Data TLB
    DTLB = perf_event::PERF_COUNT_HW_CACHE_DTLB as isize,

    /// For measuring the Instruction TLB
    ITLB = perf_event::PERF_COUNT_HW_CACHE_ITLB as isize,

    /// For measuring the branch prediction unit
    BPU = perf_event::PERF_COUNT_HW_CACHE_BPU as isize,

    /// For measuring local memory accesses
    ///
    /// (Since Linux 3.0)
    NODE = perf_event::PERF_COUNT_HW_CACHE_NODE as isize,
}

pub enum CacheOpId {
    /// For read accesses
    Read = perf_event::PERF_COUNT_HW_CACHE_OP_READ as isize,

    /// For write accesses
    Write = perf_event::PERF_COUNT_HW_CACHE_OP_WRITE as isize,

    /// For prefetch accesses
    Prefetch = perf_event::PERF_COUNT_HW_CACHE_OP_PREFETCH as isize,
}

pub enum CacheOpResultId {
    /// To measure accesses.
    Access = perf_event::PERF_COUNT_HW_CACHE_RESULT_ACCESS as isize,

    /// To measure misses.
    Miss = perf_event::PERF_COUNT_HW_CACHE_RESULT_MISS as isize,
}

pub enum ReadFormat {
    /// Adds the 64-bit time_enabled field.  This can be used to calculate estimated totals if the PMU is overcommitted
    /// and multiplexing is happening.
    TotalTimeEnabled = perf_event::PERF_FORMAT_TOTAL_TIME_ENABLED as isize,

    /// Adds the 64-bit time_running field.  This can be used to calculate estimated totals if the PMU is  overcommitted
    /// and  multiplexing is happening.
    TotalTimeRunning = perf_event::PERF_FORMAT_TOTAL_TIME_RUNNING as isize,

    /// Adds a 64-bit unique value that corresponds to the event group.
    FormatId = perf_event::PERF_FORMAT_ID as isize,

    /// Allows all counter values in an event group to be read with one read.
    FormatGroup = perf_event::PERF_FORMAT_GROUP as isize
}


impl PerfCounterBuilderLinux {

    /// Instantiate a generic performance counter for hardware events as defined by the Linux interface.
    pub fn from_hardware_event(event: HardwareEventType) -> PerfCounterBuilderLinux {
        let mut pc: PerfCounterBuilderLinux = Default::default();

        pc.attrs._type = perf_event::PERF_TYPE_HARDWARE;
        pc.attrs.config = event as u64;
        pc
    }

    /// Instantiate a generic performance counter for software events as defined by the Linux interface.
    pub fn from_software_event(event: SoftwareEventType) -> PerfCounterBuilderLinux {
        let mut pc: PerfCounterBuilderLinux = Default::default();

        pc.attrs._type = perf_event::PERF_TYPE_SOFTWARE;
        pc.attrs.config = event as u64;
        pc
    }

    /// Instantiate a generic performance counter for software events as defined by the Linux interface.
    pub fn from_cache_event(cache_id: CacheId, cache_op_id: CacheOpId, cache_op_result_id: CacheOpResultId) -> PerfCounterBuilderLinux {
        let mut pc: PerfCounterBuilderLinux = Default::default();

        pc.attrs._type = perf_event::PERF_TYPE_HW_CACHE;
        pc.attrs.config = (cache_id as u64) | (cache_op_id as u64) << 8 | (cache_op_result_id as u64) << 16;
        pc
    }

    //pub fn from_breakpoint_event() -> PerfCounterBuilderLinux {
    // NYI
    //}

    /// Instantiate a H/W performance counter using a hardware event as described in Intels SDM.
    pub fn from_intel_event_description(counter: &IntelPerformanceCounterDescription) -> PerfCounterBuilderLinux {
        let mut pc: PerfCounterBuilderLinux = Default::default();
        let mut config: u64 = 0;

        match counter.event_code {
            Tuple::One(code) =>  config |= (code as u64) << 0,
            Tuple::Two(_, _) => unreachable!() // NYI
        };
        match counter.umask {
            Tuple::One(code) =>  config |= (code as u64) << 8,
            Tuple::Two(_, _) => unreachable!() // NYI
        };
        config |= (counter.counter_mask as u64) << 24;

        if counter.edge_detect {
            config |= 1 << 18;
        }
        if counter.any_thread {
            config |= 1 << 21;
        }
        if counter.invert {
            config |= 1 << 23;
        }

        pc.attrs._type = perf_event::PERF_TYPE_RAW;
        pc.attrs.config = config;
        pc
    }

    /// Set counter group.
    pub fn set_group<'a>(&'a mut self, group_fd: isize) -> &'a mut PerfCounterBuilderLinux {
        self.group = group_fd;
        self
    }

    /// Sets PERF_FLAG_FD_OUTPUT
    ///
    /// This flag re-routes the output from an event to the group leader.
    pub fn set_flag_fd_output<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.flags |= 0x0; //PERF_FLAG_FD_OUTPUT;
        self
    }

    /// Sets PERF_FLAG_PID_CGROUP
    ///
    /// This flag activates per-container system-wide monitoring.  A
    /// container is an abstraction that isolates a set of resources for
    /// finer grain control (CPUs, memory, etc.).   In  this  mode,  the
    /// event  is  measured  only if the thread running on the monitored
    /// CPU belongs to the designated container (cgroup).
    pub fn set_flag_pid_cgroup<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.flags |= 0x0; //PERF_FLAG_PID_CGROUP;
        self
    }

    /// Add a sample period.
    pub fn set_sample_period<'a>(&'a mut self, period: u64) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.sample_period_freq = period;
        self
    }

    /// Add a sample frequency.
    pub fn set_sample_frequency<'a>(&'a mut self, frequency: u64) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.sample_period_freq = frequency;
        self.attrs.settings.insert(perf_event::EVENT_ATTR_FREQ);
        self
    }

    /// The counter starts out disabled.
    pub fn disable<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(perf_event::EVENT_ATTR_DISABLED);
        self
    }

    /// This counter should count events of child tasks as well as the task specified.
    pub fn inherit<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(perf_event::EVENT_ATTR_INHERIT);
        self
    }

    /// The pinned bit specifies that the counter should always be on the CPU if at all possible.
    /// It applies only to  hardware counters and only to group leaders.
    pub fn pinned<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(perf_event::EVENT_ATTR_PINNED);
        self
    }

    /// The counter is exclusive i.e., when this counter's group is on the CPU,
    /// it should be the only group using the CPU's counters.
    pub fn exclusive<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(perf_event::EVENT_ATTR_EXCLUSIVE);
        self
    }

    /// The counter excludes events that happen in user space.
    pub fn exclude_user<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(perf_event::EVENT_ATTR_EXCLUDE_USER);
        self
    }

    /// The counter excludes events that happen in the kernel.
    pub fn exclude_kernel<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(perf_event::EVENT_ATTR_EXCLUDE_KERNEL);
        self
    }

    /// The counter excludes events that happen in the hypervisor.
    pub fn exclude_hv<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(perf_event::EVENT_ATTR_EXCLUDE_HV);
        self
    }

    /// The counter doesn't count when the CPU is idle.
    pub fn exclude_idle<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(perf_event::EVENT_ATTR_EXCLUDE_IDLE);
        self
    }

    /// Enables recording of exec mmap events.
    pub fn enable_mmap<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(perf_event::EVENT_ATTR_MMAP);
        self
    }

    /// The counter will save event counts on context switch for inherited tasks.
    /// This is meaningful only if the inherit field is set.
    pub fn inherit_stat<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(perf_event::EVENT_ATTR_INHERIT_STAT);
        self
    }

    /// The counter is automatically enabled after a call to exec.
    pub fn enable_on_exec<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(perf_event::EVENT_ATTR_ENABLE_ON_EXEC);
        self
    }

    /// fork/exit notifications are included in the ring buffer.
    pub fn enable_task_notification<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(perf_event::EVENT_ATTR_TASK);
        self
    }

    /// The counter has  a  sampling  interrupt happen when we cross the wakeup_watermark
    /// boundary.  Otherwise interrupts happen after wakeup_events samples.
    pub fn enable_watermark<'a>(&'a mut self, watermark_events: u32) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(perf_event::EVENT_ATTR_WATERMARK);
        self.attrs.wakeup_events_watermark = watermark_events;
        self
    }

    /// Sampled IP counter can have arbitrary skid.
    pub fn set_ip_sample_arbitrary_skid<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(perf_event::EVENT_ATTR_SAMPLE_IP_ARBITRARY_SKID);
        self
    }

    /// Sampled IP counter requested to have constant skid.
    pub fn set_ip_sample_constant_skid<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(perf_event::EVENT_ATTR_SAMPLE_IP_CONSTANT_SKID);
        self
    }

    /// Sampled IP counter requested to have 0 skid.
    pub fn set_ip_sample_req_zero_skid<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(perf_event::EVENT_ATTR_SAMPLE_IP_REQ_ZERO_SKID);
        self
    }

    /// The counterpart of enable_mmap, but enables including data mmap events in the ring-buffer.
    pub fn enable_mmap_data<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(perf_event::EVENT_ATTR_MMAP_DATA);
        self
    }

    /// Sampled IP counter must have 0 skid.
    pub fn set_ip_sample_zero_skid<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(perf_event::EVENT_ATTR_SAMPLE_IP_ZERO_SKID);
        self
    }

    pub fn add_read_format<'a>(&'a mut self, flag: ReadFormat) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.read_format |= flag as u64;
        self
    }

    /// Measure for all PIDs on the core.
    pub fn for_all_pids<'a>(&'a mut self) ->  &'a mut PerfCounterBuilderLinux {
        self.pid = -1;
        self
    }

    /// Measure for a specific PID.
    pub fn for_pid<'a>(&'a mut self, pid: i32) -> &'a mut PerfCounterBuilderLinux {
        self.pid = pid;
        self
    }

    /// Pin counter to CPU.
    pub fn on_cpu<'a>(&'a mut self, cpu: isize) -> &'a mut PerfCounterBuilderLinux {
        self.cpu = cpu;
        self
    }

    /// Measure on all CPUs.
    pub fn on_all_cpus<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.cpu = -1;
        self
    }

    pub fn finish_sampling_counter(&self) -> Result<PerfCounter, io::Error> {
        let flags = 0;
        let fd = perf_event_open(self.attrs, self.pid, self.cpu as i32, self.group as i32, flags) as ::libc::c_int;
        if fd < 0 {
            return Err(Error::from_raw_os_error(-fd));
        }

        Ok(PerfCounter { fd: fd, file: unsafe { File::from_raw_fd(fd) } })
    }

    /// Instantiate the performance counter.
    pub fn finish(&self) -> Result<PerfCounter, io::Error> {
        let flags = 0;
        let fd = perf_event_open(self.attrs, self.pid, self.cpu as i32, self.group as i32, flags) as ::libc::c_int;
        if fd < 0 {
            return Err(Error::from_raw_os_error(-fd));
        }

        Ok(PerfCounter { fd: fd, file: unsafe { File::from_raw_fd(fd) } })
    }
}

#[repr(C)]
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

#[repr(C)]
pub struct MMAPPage {
    /// version number of this structure
    version: u32,
    /// lowest version this is compat with
    compat_version: u32,
    /// seqlock for synchronization
    lock: u32,
    /// hardware counter identifier
    index: u32,
    /// add to hardware counter value
    offset: i64,
    /// time event active
    time_enabled: u64,
    /// time event on CPU
    time_running: u64,
    capabilities: u64,
    pmc_width: u16,
    time_shift: u16,
    time_mult: u32,
    time_offset: u64,
    /// Pad to 1k
    reserved: [u64; 120],
    /// head in the data section
    data_head: u64,
    /// user-space written tail
    data_tail: u64,
}

impl fmt::Debug for MMAPPage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MMAPPage {{ version: {} compat_version: {} lock: {} index: {} offset: {} time_enabled: {} time_running: {} capabilities: {} pmc_width: {} time_shift: {} time_mult: {}  time_offset: {} data_head: {} data_tail: {} }}",
            self.version, self.compat_version, self.lock,
            self.index, self.offset, self.time_enabled, self.time_running,
            self.capabilities, self.pmc_width, self.time_shift, self.time_mult,
            self.time_offset, self.data_head, self.data_tail)
    }
}

pub struct PerfCounter {
    fd: ::libc::c_int,
    file: File,
}

impl PerfCounter {

    /// Read the file descriptor and parse the return format.
    pub fn read_fd(&mut self) -> Result<FileReadFormat, io::Error> {
        unsafe {
            let mut value: FileReadFormat = Default::default();
            let ptr = mem::transmute::<&mut FileReadFormat, &mut u8>(&mut value);
            let slice = slice::from_raw_parts_mut::<u8>(ptr, mem::size_of::<FileReadFormat>());
            try!(self.file.read_exact(slice));
            Ok(value)
        }
    }
}

impl<'a> AbstractPerfCounter for PerfCounter {

    fn reset(&self) -> Result<(), io::Error> {
        let ret = ioctl(self.fd, perf_event::PERF_EVENT_IOC_RESET, 0);
        if ret == -1 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }

    fn start(&self) -> Result<(), io::Error> {
        let ret = ioctl(self.fd, perf_event::PERF_EVENT_IOC_ENABLE, 0);
        if ret == -1 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }

    fn stop(&self) -> Result<(), io::Error> {
        let ret = ioctl(self.fd, perf_event::PERF_EVENT_IOC_DISABLE, 0);
        if ret == -1 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }

    fn read(&mut self) -> Result<u64, io::Error> {
        let value: FileReadFormat = try!(self.read_fd());
        return Ok(value.value)
    }
}

pub struct SamplingPerfCounter<'a> {
    pc: PerfCounter,
    map: mmap::MemoryMap,
    header: &'a MMAPPage,
    events: *const u8,
    events_size: usize
}

#[repr(C)]
#[derive(Default, Debug)]
struct EventHeader {
    event_type: u32,
    misc: u16,
    size: u16,
}

/*
enum EventHeaderMisc {

    /// Unknown CPU mode.
    CPUMODE_UNKNOWN

    /// Sample happened in the kernel.
    KERNEL

    /// Sample happened in user code.
    USER

    /// Sample happened in the hypervisor.
    HYPERVISOR

    /// Sample happened in the guest kernel.
    GUEST_KERNEL

    /// Sample happened in guest user code.
    GUEST_USER


    In addition, one of the following bits can be set:
    MMAP_DATA
           This is set when the mapping is not executable; otherwise the mapping is executable.

    EXACT_IP
           This indicates that the content of PERF_SAMPLE_IP points to the actual instruction  that  triggered  the  event.
           See also perf_event_attr.precise_ip.

    EXT_RESERVED
           This indicates there is extended data available (currently not used).

}*/


/// The MMAP events record the PROT_EXEC mappings so that we can correlate user-space IPs to code.
#[repr(C)]
#[derive(Debug)]
pub struct MMAPRecord {
    header: EventHeader,
    pid: u32,
    tid: u32,
    addr: u64,
    len: u64,
    pgoff: u64,
    filename: u8
}

impl MMAPRecord {
    pub fn filename(&self) -> Result<&str, str::Utf8Error> {
        unsafe {
            let strlen_ptr = mem::transmute::<&u8, &i8>(&self.filename);
            let length = strlen(strlen_ptr) as usize;

            let slice = slice::from_raw_parts(&self.filename, length);
            str::from_utf8(slice)
        }
    }
}

/// This record indicates when events are lost.
#[repr(C)]
#[derive(Debug)]
pub struct LostRecord {
    header: EventHeader,
    /// Unique event ID of the samples that were lost.
    id: u64,
    /// The number of events that were lost.
    lost: u64,
}

/// This record indicates a change in the process name.
#[repr(C)]
#[derive(Debug)]
pub struct CommRecord {
    header: EventHeader,
    pid: u32,
    tid: u32,
    /// Really a char[] in C
    comm: u8
}

/// This record indicates a process exit event.
#[repr(C)]
#[derive(Debug)]
pub struct ExitRecord {
    header: EventHeader,
    pid: u32,
    ppid: u32,
    tid: u32,
    ptid: u32,
    time: u64
}

/// This record indicates a throttle/unthrottle event.
#[repr(C)]
#[derive(Debug)]
pub struct ThrottleRecord {
    header: EventHeader,
    time: u64,
    id: u64,
    stream_id: u64,
}

/// This record indicates a fork event.
#[repr(C)]
#[derive(Debug)]
pub struct ForkRecord {
    header: EventHeader,
    pid: u32,
    ppid: u32,
    tid: u32,
    ptid: u32,
    time: u64,
}

/// This record indicates a read event.
#[repr(C)]
#[derive(Debug)]
pub struct ReadRecord {
    header: EventHeader,
    pid: u32,
    tid: u32,
    // TOOD: struct read_format values,
}

/// This record indicates a sample.

#[repr(C)]
#[derive(Debug)]
pub struct SampleRecord {
    header: EventHeader,

//u64   ip;         /* if PERF_SAMPLE_IP */
//u32   pid, tid;   /* if PERF_SAMPLE_TID */
//u64   time;       /* if PERF_SAMPLE_TIME */
//u64   addr;       /* if PERF_SAMPLE_ADDR */
//u64   id;         /* if PERF_SAMPLE_ID */
//u64   stream_id;  /* if PERF_SAMPLE_STREAM_ID */
//u32   cpu, res;   /* if PERF_SAMPLE_CPU */
//u64   period;     /* if PERF_SAMPLE_PERIOD */
//struct read_format v; /* if PERF_SAMPLE_READ */
//u64   nr;         /* if PERF_SAMPLE_CALLCHAIN */
//u64   ips[nr];    /* if PERF_SAMPLE_CALLCHAIN */
//u32   size;       /* if PERF_SAMPLE_RAW */
//char  data[size]; /* if PERF_SAMPLE_RAW */
//u64   bnr;        /* if PERF_SAMPLE_BRANCH_STACK */
//struct perf_branch_entry lbr[bnr];
//              /* if PERF_SAMPLE_BRANCH_STACK */
//u64   abi;        /* if PERF_SAMPLE_REGS_USER */
//u64   regs[weight(mask)];
//                  /* if PERF_SAMPLE_REGS_USER */
//u64   size;       /* if PERF_SAMPLE_STACK_USER */
//char  data[size]; /* if PERF_SAMPLE_STACK_USER */
//u64   dyn_size;   /* if PERF_SAMPLE_STACK_USER */
//u64   weight;     /* if PERF_SAMPLE_WEIGHT */
//u64   data_src;   /* if PERF_SAMPLE_DATA_SRC */
}


#[derive(Debug)]
pub enum Event<'a> {
    MMAP(&'a MMAPRecord),
    Lost(&'a LostRecord),
    Comm(&'a CommRecord),
    Exit(&'a ExitRecord),
    Throttle(&'a ThrottleRecord),
    Unthrottle(&'a ThrottleRecord),
    Fork(&'a ForkRecord),
    Read(&'a ReadRecord),
    Sample(&'a SampleRecord),
}

impl<'a> Iterator for SamplingPerfCounter<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Event<'a>> {
        if self.header.data_tail < self.header.data_head {
            let event: &EventHeader = unsafe { mem::transmute::<*const u8, &EventHeader>(self.events) };
            match event.event_type {
                perf_event::PERF_RECORD_MMAP => {
                    let mr: &MMAPRecord = unsafe { mem::transmute::<*const u8, &MMAPRecord>(self.events) };
                    Some(Event::MMAP(mr))
                },
                perf_event::PERF_RECORD_LOST => {
                    let record: &LostRecord = unsafe { mem::transmute::<*const u8, &LostRecord>(self.events) };
                    Some(Event::Lost(record))
                },
                perf_event::PERF_RECORD_COMM => {
                    let record: &CommRecord = unsafe { mem::transmute::<*const u8, &CommRecord>(self.events) };
                    Some(Event::Comm(record))
                },
                perf_event::PERF_RECORD_EXIT => {
                    let record: &ExitRecord = unsafe { mem::transmute::<*const u8, &ExitRecord>(self.events) };
                    Some(Event::Exit(record))
                },
                perf_event::PERF_RECORD_THROTTLE => {
                    let record: &ThrottleRecord = unsafe { mem::transmute::<*const u8, &ThrottleRecord>(self.events) };
                    Some(Event::Throttle(record))
                },
                perf_event::PERF_RECORD_UNTHROTTLE => {
                    let record: &ThrottleRecord = unsafe { mem::transmute::<*const u8, &ThrottleRecord>(self.events) };
                    Some(Event::Unthrottle(record))
                },
                perf_event::PERF_RECORD_FORK => {
                    let record: &ForkRecord = unsafe { mem::transmute::<*const u8, &ForkRecord>(self.events) };
                    Some(Event::Fork(record))
                },
                perf_event::PERF_RECORD_READ => {
                    let record: &ReadRecord = unsafe { mem::transmute::<*const u8, &ReadRecord>(self.events) };
                    Some(Event::Read(record))
                },
                perf_event::PERF_RECORD_SAMPLE => {
                    let record: &SampleRecord = unsafe { mem::transmute::<*const u8, &SampleRecord>(self.events) };
                    Some(Event::Sample(record))
                },
                perf_event::PERF_RECORD_MMAP2 => { unreachable!(); },
                _ => { panic!("Unknown type"); }
            }
        }
        else {
            None
        }
    }
}

impl<'a> SamplingPerfCounter<'a> {

    pub fn new(pc: PerfCounter) -> SamplingPerfCounter<'a> {
        let size = (1+16)*4096;
        let res: mmap::MemoryMap = mmap::MemoryMap::new(size,
            &[ mmap::MapOption::MapFd(pc.fd),
               mmap::MapOption::MapOffset(0),
               mmap::MapOption::MapNonStandardFlags(MAP_SHARED),
               mmap::MapOption::MapReadable ]).unwrap();

        let header = unsafe { mem::transmute::<*mut u8, &MMAPPage>(res.data()) };
        //mem::size_of::<MMAPPage>() as isize))
        let events = unsafe { mem::transmute::<*mut u8, *const u8>(res.data().offset(4096)) };

        SamplingPerfCounter{ pc: pc, map: res, header: header, events: events, events_size: 16*4096 }
    }

    pub fn print(&mut self) {
        let event: Event = self.next().unwrap();
        println!("{:?}", event);
        match event {
            Event::MMAPRecord(a) => println!("{:?}", a.filename()),
        }

    }
}
