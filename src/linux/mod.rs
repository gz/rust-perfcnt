//! A wrapper around perf_event open (http://lxr.free-electrons.com/source/tools/perf/design.txt)

use std::fmt;
use std::fs::File;
use std::io;
use std::io::{Error, Read};
use std::mem;
use std::os::unix::io::FromRawFd;
use std::ptr;
use std::slice;
use std::str;

use libc::{pid_t, strlen, MAP_SHARED};
use mmap;

#[allow(dead_code, non_camel_case_types)]
mod hw_breakpoint;
#[allow(dead_code, non_camel_case_types)]
mod perf_event;

pub mod parser;
pub mod perf_file;
pub mod perf_format;

use self::perf_format::{EventAttrFlags, ReadFormatFlags, SampleFormatFlags};

use crate::AbstractPerfCounter;

fn perf_event_open(
    hw_event: &perf_format::EventAttr,
    pid: perf_event::__kernel_pid_t,
    cpu: ::libc::c_int,
    group_fd: ::libc::c_int,
    flags: ::libc::c_int,
) -> isize {
    unsafe {
        libc::syscall(
            libc::SYS_perf_event_open,
            hw_event as *const perf_format::EventAttr as usize,
            pid,
            cpu,
            group_fd,
            flags
        ) as isize
    }
}

fn ioctl(fd: ::libc::c_int, request: u64, value: ::libc::c_int) -> isize {
    unsafe { libc::ioctl(fd, request, value) as isize }
}

pub struct PerfCounterBuilderLinux {
    group: isize,
    pid: pid_t,
    cpu: isize,
    flags: i32,
    attrs: perf_format::EventAttr,
}

impl Default for PerfCounterBuilderLinux {
    fn default() -> PerfCounterBuilderLinux {
        PerfCounterBuilderLinux {
            group: -1,
            pid: 0,
            cpu: -1,
            flags: 0,
            attrs: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HardwareEventType {
    /// Total cycles.  Be wary of what happens during CPU frequency scaling.
    CPUCycles = perf_event::PERF_COUNT_HW_CPU_CYCLES as isize,

    /// Retired instructions.  Be careful, these can be affected by various issues, most notably
    /// hardware interrupt counts.
    Instructions = perf_event::PERF_COUNT_HW_INSTRUCTIONS as isize,

    /// Cache accesses. Usually this indicates Last Level Cache accesses but this may vary depending
    /// on your CPU. This may include prefetches and coherency messages; again this depends on the
    /// design of your CPU.
    CacheReferences = perf_event::PERF_COUNT_HW_CACHE_REFERENCES as isize,

    /// Cache misses. Usually this indicates Last Level Cache misses; this is intended to be used in
    /// conjunction with the [CacheReferences] event to calculate cache miss rates.
    CacheMisses = perf_event::PERF_COUNT_HW_CACHE_MISSES as isize,

    /// Retired branch instructions.  Prior to Linux 2.6.34, this used the wrong event on AMD
    /// processors.
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum CacheOpId {
    /// For read accesses
    Read = perf_event::PERF_COUNT_HW_CACHE_OP_READ as isize,

    /// For write accesses
    Write = perf_event::PERF_COUNT_HW_CACHE_OP_WRITE as isize,

    /// For prefetch accesses
    Prefetch = perf_event::PERF_COUNT_HW_CACHE_OP_PREFETCH as isize,
}

#[derive(Debug, Clone, Copy)]
pub enum CacheOpResultId {
    /// To measure accesses.
    Access = perf_event::PERF_COUNT_HW_CACHE_RESULT_ACCESS as isize,

    /// To measure misses.
    Miss = perf_event::PERF_COUNT_HW_CACHE_RESULT_MISS as isize,
}

impl PerfCounterBuilderLinux {
    /// Instantiate a generic performance counter for hardware events as defined by the Linux interface.
    pub fn from_hardware_event(event: HardwareEventType) -> PerfCounterBuilderLinux {
        let mut pc: PerfCounterBuilderLinux = Default::default();

        pc.attrs.attr_type = perf_event::PERF_TYPE_HARDWARE;
        pc.attrs.config = event as u64;
        pc
    }

    /// Instantiate a generic performance counter for software events as defined by the Linux interface.
    pub fn from_software_event(event: SoftwareEventType) -> PerfCounterBuilderLinux {
        let mut pc: PerfCounterBuilderLinux = Default::default();

        pc.attrs.attr_type = perf_event::PERF_TYPE_SOFTWARE;
        pc.attrs.config = event as u64;
        pc
    }

    /// Instantiate a generic performance counter for software events as defined by the Linux interface.
    pub fn from_cache_event(
        cache_id: CacheId,
        cache_op_id: CacheOpId,
        cache_op_result_id: CacheOpResultId,
    ) -> PerfCounterBuilderLinux {
        let mut pc: PerfCounterBuilderLinux = Default::default();

        pc.attrs.attr_type = perf_event::PERF_TYPE_HW_CACHE;
        pc.attrs.config =
            (cache_id as u64) | (cache_op_id as u64) << 8 | (cache_op_result_id as u64) << 16;
        pc
    }

    //pub fn from_breakpoint_event() -> PerfCounterBuilderLinux {
    // NYI
    //}

    /// Instantiate a H/W performance counter using a hardware event as described in Intels SDM.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub fn from_intel_event_description(counter: &x86::perfcnt::intel::EventDescription) -> PerfCounterBuilderLinux {
        use x86::perfcnt::intel::Tuple;
        let mut pc: PerfCounterBuilderLinux = Default::default();
        let mut config: u64 = 0;

        match counter.event_code {
            Tuple::One(code) => config |= (code as u64) << 0,
            Tuple::Two(_, _) => unreachable!(), // NYI
        };
        match counter.umask {
            Tuple::One(code) => config |= (code as u64) << 8,
            Tuple::Two(_, _) => unreachable!(), // NYI
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

        pc.attrs.attr_type = perf_event::PERF_TYPE_RAW;
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
        self.flags |= 0x02; //PERF_FLAG_FD_OUTPUT;
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
        self.flags |= 0x04; //PERF_FLAG_PID_CGROUP;
        self
    }

    /// Add a sample period.
    pub fn set_sample_period<'a>(&'a mut self, period: u64) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.sample_period_freq = period;
        self
    }

    /// Add a sample frequency.
    pub fn set_sample_frequency<'a>(
        &'a mut self,
        frequency: u64,
    ) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.sample_period_freq = frequency;
        self.attrs.settings.insert(EventAttrFlags::EVENT_ATTR_FREQ);
        self
    }

    /// The counter starts out disabled.
    pub fn disable<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs
            .settings
            .insert(EventAttrFlags::EVENT_ATTR_DISABLED);
        self
    }

    /// This counter should count events of child tasks as well as the task specified.
    pub fn inherit<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs
            .settings
            .insert(EventAttrFlags::EVENT_ATTR_INHERIT);
        self
    }

    /// The pinned bit specifies that the counter should always be on the CPU if at all possible.
    /// It applies only to  hardware counters and only to group leaders.
    pub fn pinned<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs
            .settings
            .insert(EventAttrFlags::EVENT_ATTR_PINNED);
        self
    }

    /// The counter is exclusive i.e., when this counter's group is on the CPU,
    /// it should be the only group using the CPU's counters.
    pub fn exclusive<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs
            .settings
            .insert(EventAttrFlags::EVENT_ATTR_EXCLUSIVE);
        self
    }

    /// The counter excludes events that happen in user space.
    pub fn exclude_user<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs
            .settings
            .insert(EventAttrFlags::EVENT_ATTR_EXCLUDE_USER);
        self
    }

    /// The counter excludes events that happen in the kernel.
    pub fn exclude_kernel<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs
            .settings
            .insert(EventAttrFlags::EVENT_ATTR_EXCLUDE_KERNEL);
        self
    }

    /// The counter excludes events that happen in the hypervisor.
    pub fn exclude_hv<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs
            .settings
            .insert(EventAttrFlags::EVENT_ATTR_EXCLUDE_HV);
        self
    }

    /// The counter doesn't count when the CPU is idle.
    pub fn exclude_idle<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs
            .settings
            .insert(EventAttrFlags::EVENT_ATTR_EXCLUDE_IDLE);
        self
    }

    /// Enables recording of exec mmap events.
    pub fn enable_mmap<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(EventAttrFlags::EVENT_ATTR_MMAP);
        self
    }

    /// The counter will save event counts on context switch for inherited tasks.
    /// This is meaningful only if the inherit field is set.
    pub fn inherit_stat<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs
            .settings
            .insert(EventAttrFlags::EVENT_ATTR_INHERIT_STAT);
        self
    }

    /// The counter is automatically enabled after a call to exec.
    pub fn enable_on_exec<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs
            .settings
            .insert(EventAttrFlags::EVENT_ATTR_ENABLE_ON_EXEC);
        self
    }

    /// fork/exit notifications are included in the ring buffer.
    pub fn enable_task_notification<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.settings.insert(EventAttrFlags::EVENT_ATTR_TASK);
        self
    }

    /// The counter has  a  sampling  interrupt happen when we cross the wakeup_watermark
    /// boundary.  Otherwise interrupts happen after wakeup_events samples.
    pub fn enable_watermark<'a>(
        &'a mut self,
        watermark_events: u32,
    ) -> &'a mut PerfCounterBuilderLinux {
        self.attrs
            .settings
            .insert(EventAttrFlags::EVENT_ATTR_WATERMARK);
        self.attrs.wakeup_events_watermark = watermark_events;
        self
    }

    /// Sampled IP counter can have arbitrary skid.
    pub fn set_ip_sample_arbitrary_skid<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs
            .settings
            .insert(EventAttrFlags::EVENT_ATTR_SAMPLE_IP_ARBITRARY_SKID);
        self
    }

    /// Sampled IP counter requested to have constant skid.
    pub fn set_ip_sample_constant_skid<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs
            .settings
            .insert(EventAttrFlags::EVENT_ATTR_SAMPLE_IP_CONSTANT_SKID);
        self
    }

    /// Sampled IP counter requested to have 0 skid.
    pub fn set_ip_sample_req_zero_skid<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs
            .settings
            .insert(EventAttrFlags::EVENT_ATTR_SAMPLE_IP_REQ_ZERO_SKID);
        self
    }

    /// The counterpart of enable_mmap, but enables including data mmap events in the ring-buffer.
    pub fn enable_mmap_data<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs
            .settings
            .insert(EventAttrFlags::EVENT_ATTR_MMAP_DATA);
        self
    }

    /// Sampled IP counter must have 0 skid.
    pub fn set_ip_sample_zero_skid<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs
            .settings
            .insert(EventAttrFlags::EVENT_ATTR_SAMPLE_IP_ZERO_SKID);
        self
    }

    /// Adds the 64-bit time_enabled field.  This can be used to calculate estimated totals if the PMU is overcommitted
    /// and multiplexing is happening.
    pub fn enable_read_format_time_enabled<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs
            .read_format
            .insert(ReadFormatFlags::FORMAT_TOTAL_TIME_ENABLED);
        self
    }

    /// Adds the 64-bit time_running field.  This can be used to calculate estimated totals if the PMU is  overcommitted
    /// and  multiplexing is happening.
    pub fn enable_read_format_time_running<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs
            .read_format
            .insert(ReadFormatFlags::FORMAT_TOTAL_TIME_RUNNING);
        self
    }

    /// Adds a 64-bit unique value that corresponds to the event group.
    pub fn enable_read_format_id<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.read_format.insert(ReadFormatFlags::FORMAT_ID);
        self
    }

    /// Allows all counter values in an event group to be read with one read.
    pub fn enable_read_format_group<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
        self.attrs.read_format.insert(ReadFormatFlags::FORMAT_GROUP);
        self
    }

    pub fn enable_sampling_ip<'a>(&'a mut self) -> &'a PerfCounterBuilderLinux {
        self.attrs
            .sample_type
            .insert(SampleFormatFlags::PERF_SAMPLE_IP);
        self
    }

    pub fn enable_sampling_tid<'a>(&'a mut self) -> &'a PerfCounterBuilderLinux {
        self.attrs
            .sample_type
            .insert(SampleFormatFlags::PERF_SAMPLE_TID);
        self
    }

    pub fn enable_sampling_time<'a>(&'a mut self) -> &'a PerfCounterBuilderLinux {
        self.attrs
            .sample_type
            .insert(SampleFormatFlags::PERF_SAMPLE_TIME);
        self
    }

    pub fn enable_sampling_addr<'a>(&'a mut self) -> &'a PerfCounterBuilderLinux {
        self.attrs
            .sample_type
            .insert(SampleFormatFlags::PERF_SAMPLE_ADDR);
        self
    }

    pub fn enable_sampling_read<'a>(&'a mut self) -> &'a PerfCounterBuilderLinux {
        self.attrs
            .sample_type
            .insert(SampleFormatFlags::PERF_SAMPLE_READ);
        self
    }

    pub fn enable_sampling_callchain<'a>(&'a mut self) -> &'a PerfCounterBuilderLinux {
        self.attrs
            .sample_type
            .insert(SampleFormatFlags::PERF_SAMPLE_CALLCHAIN);
        self
    }

    pub fn enable_sampling_sample_id<'a>(&'a mut self) -> &'a PerfCounterBuilderLinux {
        self.attrs
            .sample_type
            .insert(SampleFormatFlags::PERF_SAMPLE_ID);
        self
    }

    pub fn enable_sampling_cpu<'a>(&'a mut self) -> &'a PerfCounterBuilderLinux {
        self.attrs
            .sample_type
            .insert(SampleFormatFlags::PERF_SAMPLE_CPU);
        self
    }

    pub fn enable_sampling_period<'a>(&'a mut self) -> &'a PerfCounterBuilderLinux {
        self.attrs
            .sample_type
            .insert(SampleFormatFlags::PERF_SAMPLE_PERIOD);
        self
    }

    pub fn enable_sampling_stream_id<'a>(&'a mut self) -> &'a PerfCounterBuilderLinux {
        self.attrs
            .sample_type
            .insert(SampleFormatFlags::PERF_SAMPLE_STREAM_ID);
        self
    }

    pub fn enable_sampling_raw<'a>(&'a mut self) -> &'a PerfCounterBuilderLinux {
        self.attrs
            .sample_type
            .insert(SampleFormatFlags::PERF_SAMPLE_RAW);
        self
    }

    pub fn enable_sampling_branch_stack<'a>(&'a mut self) -> &'a PerfCounterBuilderLinux {
        self.attrs
            .sample_type
            .insert(SampleFormatFlags::PERF_SAMPLE_BRANCH_STACK);
        self
    }

    pub fn enable_sampling_regs_user<'a>(&'a mut self) -> &'a PerfCounterBuilderLinux {
        self.attrs
            .sample_type
            .insert(SampleFormatFlags::PERF_SAMPLE_REGS_USER);
        self
    }

    pub fn enable_sampling_stack_user<'a>(&'a mut self) -> &'a PerfCounterBuilderLinux {
        self.attrs
            .sample_type
            .insert(SampleFormatFlags::PERF_SAMPLE_STACK_USER);
        self
    }

    pub fn enable_sampling_sample_weight<'a>(&'a mut self) -> &'a PerfCounterBuilderLinux {
        self.attrs
            .sample_type
            .insert(SampleFormatFlags::PERF_SAMPLE_WEIGHT);
        self
    }

    pub fn enable_sampling_data_src<'a>(&'a mut self) -> &'a PerfCounterBuilderLinux {
        self.attrs
            .sample_type
            .insert(SampleFormatFlags::PERF_SAMPLE_DATA_SRC);
        self
    }

    pub fn enable_sampling_identifier<'a>(&'a mut self) -> &'a PerfCounterBuilderLinux {
        self.attrs
            .sample_type
            .insert(SampleFormatFlags::PERF_SAMPLE_IDENTIFIER);
        self
    }

    pub fn enable_sampling_transaction<'a>(&'a mut self) -> &'a PerfCounterBuilderLinux {
        self.attrs
            .sample_type
            .insert(SampleFormatFlags::PERF_SAMPLE_TRANSACTION);
        self
    }

    /// Measure for all PIDs on the core.
    pub fn for_all_pids<'a>(&'a mut self) -> &'a mut PerfCounterBuilderLinux {
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
        let fd = perf_event_open(
            &self.attrs,
            self.pid,
            self.cpu as i32,
            self.group as i32,
            self.flags,
        ) as ::libc::c_int;
        if fd < 0 {
            return Err(Error::from_raw_os_error(-fd));
        }

        Ok(PerfCounter {
            fd,
            file: unsafe { File::from_raw_fd(fd) },
            attributes: self.attrs,
        })
    }

    /// Instantiate the performance counter.
    pub fn finish(&self) -> Result<PerfCounter, io::Error> {
        let fd = perf_event_open(
            &self.attrs,
            self.pid,
            self.cpu as i32,
            self.group as i32,
            self.flags,
        ) as ::libc::c_int;
        if fd < 0 {
            return Err(Error::from_raw_os_error(-fd));
        }

        Ok(PerfCounter {
            fd,
            file: unsafe { File::from_raw_fd(fd) },
            attributes: self.attrs,
        })
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

impl FileReadFormat {
    unsafe fn copy_from_raw_ptr(ptr: *const u8) -> FileReadFormat {
        let value: u64 = read(ptr, 0);
        let time_enabled: u64 = read(ptr, 8);
        let time_running: u64 = read(ptr, 16);
        let id: u64 = read(ptr, 24);

        FileReadFormat {
            value,
            time_enabled,
            time_running,
            id,
        }
    }
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
    attributes: perf_format::EventAttr,
}

impl PerfCounter {
    /// Read the file descriptor and parse the return format.
    pub fn read_fd(&mut self) -> Result<FileReadFormat, io::Error> {
        unsafe {
            let mut value: FileReadFormat = Default::default();
            let ptr = mem::transmute::<&mut FileReadFormat, &mut u8>(&mut value);
            let slice = slice::from_raw_parts_mut::<u8>(ptr, mem::size_of::<FileReadFormat>());
            self.file.read_exact(slice)?;
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
        let value: FileReadFormat = self.read_fd()?;
        return Ok(value.value);
    }
}

pub struct SamplingPerfCounter {
    pc: PerfCounter,
    map: mmap::MemoryMap,
    events_size: usize,
}

unsafe fn read<U: Copy>(ptr: *const u8, offset: isize) -> U {
    let newptr = mem::transmute::<*const u8, *const U>(ptr.offset(offset));
    ptr::read(newptr)
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

#[derive(Default, Debug)]
struct EventHeader {
    event_type: u32,
    misc: u16,
    size: u16,
}

impl EventHeader {
    unsafe fn copy_from_raw_ptr(ptr: *const u8) -> EventHeader {
        let event_type: u32 = read(ptr, 0);
        let misc: u16 = read(ptr, 4);
        let size: u16 = read(ptr, 6);
        EventHeader {
            event_type,
            misc,
            size,
        }
    }
}

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
    filename: String,
}

impl MMAPRecord {
    unsafe fn copy_from_raw_ptr(ptr: *const u8) -> MMAPRecord {
        let header: EventHeader = EventHeader::copy_from_raw_ptr(ptr);
        let pid: u32 = read(ptr, 8);
        let tid: u32 = read(ptr, 12);
        let addr: u64 = read(ptr, 16);
        let len: u64 = read(ptr, 24);
        let pgoff: u64 = read(ptr, 32);
        let filename = {
            let str_start = ptr.offset(40);
            let strlen_ptr = str_start as *const libc::c_char;
            let length = strlen(strlen_ptr) as usize;
            let slice = slice::from_raw_parts(str_start, length);
            String::from(str::from_utf8(slice).unwrap())
        };

        MMAPRecord {
            header,
            pid,
            tid,
            addr,
            len,
            pgoff,
            filename,
        }
    }
}

/// This record indicates when events are lost.
#[derive(Debug)]
pub struct LostRecord {
    header: EventHeader,
    /// Unique event ID of the samples that were lost.
    id: u64,
    /// The number of events that were lost.
    lost: u64,
}

impl LostRecord {
    unsafe fn copy_from_raw_ptr(ptr: *const u8) -> LostRecord {
        let header: EventHeader = EventHeader::copy_from_raw_ptr(ptr);
        let id: u64 = read(ptr, 8);
        let lost: u64 = read(ptr, 16);

        LostRecord {
            header,
            id,
            lost,
        }
    }
}

/// This record indicates a change in the process name.
#[derive(Debug)]
pub struct CommRecord {
    header: EventHeader,
    pid: u32,
    tid: u32,
    comm: String,
}

impl CommRecord {
    unsafe fn copy_from_raw_ptr(ptr: *const u8) -> CommRecord {
        let header: EventHeader = EventHeader::copy_from_raw_ptr(ptr);
        let pid: u32 = read(ptr, 8);
        let tid: u32 = read(ptr, 12);

        let comm = {
            let str_start = ptr.offset(16);
            let strlen_ptr = str_start as *const libc::c_char;
            let length = strlen(strlen_ptr) as usize;
            let slice = slice::from_raw_parts(str_start, length);
            String::from(str::from_utf8(slice).unwrap())
        };
        CommRecord {
            header,
            pid,
            tid,
            comm,
        }
    }
}

/// This record indicates a process exit event.
#[derive(Debug)]
pub struct ExitRecord {
    header: EventHeader,
    pid: u32,
    ppid: u32,
    tid: u32,
    ptid: u32,
    time: u64,
}

impl ExitRecord {
    unsafe fn copy_from_raw_ptr(ptr: *const u8) -> ExitRecord {
        let header: EventHeader = EventHeader::copy_from_raw_ptr(ptr);
        let pid: u32 = read(ptr, 8);
        let ppid: u32 = read(ptr, 12);
        let tid: u32 = read(ptr, 16);
        let ptid: u32 = read(ptr, 20);
        let time: u64 = read(ptr, 24);

        ExitRecord {
            header,
            pid,
            ppid,
            tid,
            ptid,
            time,
        }
    }
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

impl ThrottleRecord {
    unsafe fn copy_from_raw_ptr(ptr: *const u8) -> ThrottleRecord {
        let header: EventHeader = EventHeader::copy_from_raw_ptr(ptr);
        let time: u64 = read(ptr, 8);
        let id: u64 = read(ptr, 16);
        let stream_id: u64 = read(ptr, 24);

        ThrottleRecord {
            header,
            time,
            id,
            stream_id,
        }
    }
}

/// This record indicates a fork event.
#[derive(Debug)]
pub struct ForkRecord {
    header: EventHeader,
    pid: u32,
    ppid: u32,
    tid: u32,
    ptid: u32,
    time: u64,
}

impl ForkRecord {
    unsafe fn copy_from_raw_ptr(ptr: *const u8) -> ForkRecord {
        let header: EventHeader = EventHeader::copy_from_raw_ptr(ptr);
        let pid: u32 = read(ptr, 8);
        let ppid: u32 = read(ptr, 12);
        let tid: u32 = read(ptr, 16);
        let ptid: u32 = read(ptr, 20);
        let time: u64 = read(ptr, 24);

        ForkRecord {
            header,
            pid,
            ppid,
            tid,
            ptid,
            time,
        }
    }
}

/// This record indicates a read event.
#[repr(C)]
#[derive(Debug)]
pub struct ReadRecord {
    header: EventHeader,
    pid: u32,
    tid: u32,
    value: FileReadFormat, // TODO with PERF_FORMAT_GROUP: values: Vec<FileReadFormat>
}

impl ReadRecord {
    unsafe fn copy_from_raw_ptr(ptr: *const u8) -> ReadRecord {
        let header: EventHeader = EventHeader::copy_from_raw_ptr(ptr);
        let pid: u32 = read(ptr, 8);
        let tid: u32 = read(ptr, 12);
        let frf: FileReadFormat = FileReadFormat::copy_from_raw_ptr(ptr.offset(16));

        ReadRecord {
            header,
            pid,
            tid,
            value: frf,
        }
    }
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
    header: EventHeader,
    /// if PERF_SAMPLE_IP
    ip: u64,
    /// if PERF_SAMPLE_TID
    pid: u32,
    /// if PERF_SAMPLE_TID
    tid: u32,
    /// if PERF_SAMPLE_TIME
    time: u64,
    /// if PERF_SAMPLE_ADDR
    addr: u64,
    /// if PERF_SAMPLE_ID
    id: u64,
    /// if PERF_SAMPLE_STREAM_ID
    stream_id: u64,
    /// if PERF_SAMPLE_CPU
    cpu: u32,
    /// if PERF_SAMPLE_CPU
    res: u32,
    /// if PERF_SAMPLE_PERIOD
    period: u64,

    /// if PERF_SAMPLE_READ
    /// # TODO
    /// FILE GROUP FORMAT is different...
    v: FileReadFormat,

    //u64   nr;         /* if PERF_SAMPLE_CALLCHAIN */
    //u64   ips[nr];    /* if PERF_SAMPLE_CALLCHAIN */
    ips: Vec<u64>,

    /// u32   size;       /* if PERF_SAMPLE_RAW */
    /// char  data[size]; /* if PERF_SAMPLE_RAW */
    raw_sample: Vec<u8>,

    /// u64   bnr;        /* if PERF_SAMPLE_BRANCH_STACK */
    /// struct perf_branch_entry lbr[bnr];
    lbr: Vec<BranchEntry>,

    /// u64   abi;        /* if PERF_SAMPLE_REGS_USER */
    abi: u64,

    ///  u64   regs[weight(mask)];
    /// if PERF_SAMPLE_REGS_USER
    regs: Vec<u64>,

    /// u64   size;       /* if PERF_SAMPLE_STACK_USER */
    /// char  data[size]; /* if PERF_SAMPLE_STACK_USER */
    user_stack: Vec<u8>,

    /// u64   dyn_size;   /* if PERF_SAMPLE_STACK_USER */
    dyn_size: u64,
    /// u64   weight;     /* if PERF_SAMPLE_WEIGHT */
    weight: u64,
    /// u64   data_src;   /* if PERF_SAMPLE_DATA_SRC */
    data_str: u64,
}

impl SampleRecord {
    unsafe fn copy_from_raw_ptr(ptr: *const u8) -> SampleRecord {
        let header: EventHeader = EventHeader::copy_from_raw_ptr(ptr);
        let ip: u64 = read(ptr, 8);
        let pid: u32 = read(ptr, 16);
        let tid: u32 = read(ptr, 20);
        let time: u64 = read(ptr, 24);
        let addr: u64 = read(ptr, 32);
        let id: u64 = read(ptr, 40);
        let stream_id: u64 = read(ptr, 48);
        let cpu: u32 = read(ptr, 52);
        let res: u32 = read(ptr, 56);
        let period: u64 = read(ptr, 64);

        // TODO:
        let v: FileReadFormat = FileReadFormat::copy_from_raw_ptr(ptr.offset(72));
        let ips: Vec<u64> = Vec::new();
        let raw_sample: Vec<u8> = Vec::new();
        let lbr: Vec<BranchEntry> = Vec::new();
        let abi: u64 = 0;
        let regs: Vec<u64> = Vec::new();
        let user_stack: Vec<u8> = Vec::new();
        let dyn_size: u64 = 0;
        let weight: u64 = 0;
        let data_str: u64 = 0;

        SampleRecord {
            header,
            ip,
            pid,
            tid,
            time,
            addr,
            id,
            stream_id,
            cpu,
            res,
            period,
            v,
            ips,
            raw_sample,
            lbr,
            abi,
            regs,
            user_stack,
            dyn_size,
            weight,
            data_str,
        }
    }
}

#[derive(Debug)]
pub enum Event {
    MMAP(MMAPRecord),
    Lost(LostRecord),
    Comm(CommRecord),
    Exit(ExitRecord),
    Throttle(ThrottleRecord),
    Unthrottle(ThrottleRecord),
    Fork(ForkRecord),
    Read(ReadRecord),
    Sample(SampleRecord),
}

impl Iterator for SamplingPerfCounter {
    type Item = Event;

    /// Iterate over the event buffer.
    ///
    /// We copy and transform the events for two reasons:
    ///  * The exposed C struct layout would be difficult to read with request.
    ///  * We need to advance the tail pointer to make space for new events.
    fn next(&mut self) -> Option<Event> {
        if self.header().data_tail < self.header().data_head {
            let offset: isize = (self.header().data_tail as usize % self.events_size) as isize;

            let mut bytes_read = 0;
            let event_ptr = unsafe { self.events().offset(offset) };
            let event: EventHeader = unsafe { EventHeader::copy_from_raw_ptr(event_ptr) };
            bytes_read += mem::size_of::<EventHeader>() as u64;

            let record = match event.event_type {
                perf_event::PERF_RECORD_MMAP => {
                    let record: MMAPRecord = unsafe { MMAPRecord::copy_from_raw_ptr(event_ptr) };
                    Some(Event::MMAP(record))
                }
                perf_event::PERF_RECORD_LOST => {
                    let record: LostRecord = unsafe { LostRecord::copy_from_raw_ptr(event_ptr) };
                    Some(Event::Lost(record))
                }
                perf_event::PERF_RECORD_COMM => {
                    let record: CommRecord = unsafe { CommRecord::copy_from_raw_ptr(event_ptr) };
                    Some(Event::Comm(record))
                }
                perf_event::PERF_RECORD_EXIT => {
                    let record: ExitRecord = unsafe { ExitRecord::copy_from_raw_ptr(event_ptr) };
                    Some(Event::Exit(record))
                }
                perf_event::PERF_RECORD_THROTTLE => {
                    let record: ThrottleRecord =
                        unsafe { ThrottleRecord::copy_from_raw_ptr(event_ptr) };
                    Some(Event::Throttle(record))
                }
                perf_event::PERF_RECORD_UNTHROTTLE => {
                    let record: ThrottleRecord =
                        unsafe { ThrottleRecord::copy_from_raw_ptr(event_ptr) };
                    Some(Event::Unthrottle(record))
                }
                perf_event::PERF_RECORD_FORK => {
                    let record: ForkRecord = unsafe { ForkRecord::copy_from_raw_ptr(event_ptr) };
                    Some(Event::Fork(record))
                }
                perf_event::PERF_RECORD_READ => {
                    let record: ReadRecord = unsafe { ReadRecord::copy_from_raw_ptr(event_ptr) };
                    Some(Event::Read(record))
                }
                perf_event::PERF_RECORD_SAMPLE => {
                    let record: SampleRecord =
                        unsafe { SampleRecord::copy_from_raw_ptr(event_ptr) };
                    Some(Event::Sample(record))
                }
                perf_event::PERF_RECORD_MMAP2 => {
                    // XXX: Not described in the man page?
                    unreachable!();
                }
                _ => {
                    panic!("Unknown type!");
                }
            };

            //bytes_read += size;

            let header = self.mut_header();
            header.data_tail = bytes_read;

            record
        } else {
            None
        }
    }
}

impl SamplingPerfCounter {
    pub fn new(pc: PerfCounter) -> SamplingPerfCounter {
        let size = (1 + 16) * 4096;
        let res: mmap::MemoryMap = mmap::MemoryMap::new(
            size,
            &[
                mmap::MapOption::MapFd(pc.fd),
                mmap::MapOption::MapOffset(0),
                mmap::MapOption::MapNonStandardFlags(MAP_SHARED),
                mmap::MapOption::MapReadable,
            ],
        )
        .unwrap();

        SamplingPerfCounter {
            pc,
            map: res,
            events_size: 16 * 4096,
        }
    }

    fn header(&self) -> &MMAPPage {
        unsafe { mem::transmute::<*mut u8, &MMAPPage>(self.map.data()) }
    }

    fn mut_header(&mut self) -> &mut MMAPPage {
        unsafe { mem::transmute::<*mut u8, &mut MMAPPage>(self.map.data()) }
    }

    fn events(&self) -> *const u8 {
        unsafe { self.map.data().offset(4096) }
    }

    pub fn print(&mut self) {
        let event: Event = self.next().unwrap();
        println!("{:?}", event);
        match event {
            Event::MMAP(a) => println!("{:?}", a.filename),
            Event::Lost(a) => println!("{:?}", a),
            Event::Comm(a) => println!("{:?}", a),
            Event::Exit(a) => println!("{:?}", a),
            Event::Throttle(a) => println!("{:?}", a),
            Event::Unthrottle(a) => println!("{:?}", a),
            Event::Fork(a) => println!("{:?}", a),
            Event::Read(a) => println!("{:?}", a),
            Event::Sample(a) => println!("{:?}", a),
        }
    }
}
