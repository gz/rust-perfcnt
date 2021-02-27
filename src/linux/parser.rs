//! Uses the `nom` library to parse the in memory format of perf data structures and
//! transforms them into more rust-like data-strutures.
//!
//! # References
//! The code is inspired by the following articles and existing parser to make sense of the
//! (poorly documented) format:
//!
//!   * https://lwn.net/Articles/644919/
//!   * http://man7.org/linux/man-pages/man2/perf_event_open.2.html
//!   * https://github.com/andikleen/pmu-tools/tree/master/parser
//!
//! # Current limitations
//!  * Only version 2 of the data format
//!  * No support for AUX stuff
//!  * Sample ID at the end of records is currently ignored
//!  * I'm not sure if I'm parsing the BuildId correctly, it seems it can not be recognized
//!  * Only support little endian machines
//!
//! # See also
//!   * `perf_file.rs` -- as an example on how to use the parser function to parse a perf.data file
//!   * `perf_format.rs` -- for all the struct definitions that are parsed here
//!

use super::perf_format::*;
use nom::*;

fn is_nul_byte(c: u8) -> bool {
    c == 0x0
}

named!(parse_c_string, take_till!(is_nul_byte));

named!(parse_vec_u64<&[u8], Vec<u64> >,
    do_parse!(
        len: le_u64 >>
        vec: count!(le_u64, len as usize) >>
        (vec)
    )
);

named!(parse_vec_u32_u8<&[u8], Vec<u8> >,
    do_parse!(
        len: le_u32 >>
        vec: count!(le_u8, len as usize) >>
        (vec)
    )
);

fn parse_vec_u64_variable(input: &[u8], count: usize) -> IResult<&[u8], Vec<u64>> {
    count!(input, le_u64, count)
}

fn parse_vec_u8_variable(input: &[u8], count: usize) -> IResult<&[u8], Vec<u8>> {
    count!(input, le_u8, count)
}

fn no_event(input: &[u8]) -> IResult<&[u8], EventData> {
    Ok((input, EventData::None))
}

// TODO: Needs sample flags!
named!(pub parse_sample_id<&[u8], SampleId>,
    do_parse!(
        ptid: parse_thread_id >>
        time: le_u64 >>
        id: le_u64 >>
        stream_id: le_u64 >>
        cpu: parse_cpu >>
        identifier: le_u64 >>
        (SampleId {
            ptid: ptid,
            time: time,
            id: id,
            stream_id: stream_id,
            cpu: cpu,
            identifier: identifier
        })
    )
);

named!(pub parse_thread_id<&[u8], ThreadId>,
    do_parse!(
        pid: le_i32 >>
        tid: le_i32 >>
        (ThreadId { pid: pid, tid: tid })
    )
);

named!(pub parse_cpu<&[u8], Cpu>,
    do_parse!(
        cpu: le_u32 >>
        res: le_u32 >>
        (Cpu { cpu: cpu, res: res })
    )
);

named!(pub parse_fork_record<&[u8], ForkRecord>,
    do_parse!(
        pid: le_u32 >>
        ppid: le_u32 >>
        tid: le_u32 >>
        ptid: le_u32 >>
        time: le_u64 >>
        (ForkRecord {
            pid: pid,
            ppid: ppid,
            tid: tid,
            ptid: ptid,
            time: time,
        })
    )
);

named!(pub parse_exit_record<&[u8], ExitRecord>,
    do_parse!(
        pid: le_u32 >>
        ppid: le_u32 >>
        tid: le_u32 >>
        ptid: le_u32 >>
        time: le_u64 >>
        (ExitRecord {
            pid: pid,
            ppid: ppid,
            tid: tid,
            ptid: ptid,
            time: time,
        })
    )
);

named!(pub parse_throttle_record<&[u8], ThrottleRecord>,
    do_parse!(
        time: le_u64 >>
        id: le_u64 >>
        stream_id: le_u64 >>
        (ThrottleRecord {
            time: time,
            id: id,
            stream_id: stream_id,
        })
    )
);

named!(pub parse_unthrottle_record<&[u8], UnthrottleRecord>,
    do_parse!(
        time: le_u64 >>
        id: le_u64 >>
        stream_id: le_u64 >>
        (UnthrottleRecord {
            time: time,
            id: id,
            stream_id: stream_id,
        })
    )
);

named!(pub parse_event_header<&[u8], EventHeader>,
    do_parse!(
        event_type: le_u32 >>
        misc: le_u16 >>
        size: le_u16 >>
        (EventHeader { event_type: EventType::new(event_type), misc: misc, size: size })
    )
);

named!(pub parse_mmap_record<&[u8], MMAPRecord>,
    do_parse!(
        pid: le_i32 >>
        tid: le_u32 >>
        addr: le_u64 >>
        len: le_u64 >>
        pgoff: le_u64 >>
        filename: parse_c_string >>
        (MMAPRecord {
            pid: pid,
            tid: tid,
            addr: addr,
            len: len,
            pgoff: pgoff,
            filename: unsafe { String::from_utf8_unchecked(filename.to_vec()) }
        })
    )
);

named!(pub parse_mmap2_record<&[u8], MMAP2Record>,
    do_parse!(
        ptid: parse_thread_id >>
        addr: le_u64 >>
        len: le_u64 >>
        pgoff: le_u64 >>
        maj: le_u32 >>
        min: le_u32 >>
        ino: le_u64 >>
        ino_generation: le_u64 >>
        prot: le_u32 >>
        flags: le_u32 >>
        filename: parse_c_string >>
        // TODO: sample_id: parse_sample_id,
        (MMAP2Record {
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

pub fn parse_read_value(
    input: &[u8],
    flags: ReadFormatFlags,
) -> IResult<&[u8], (u64, Option<u64>)> {
    do_parse!(
        input,
        value: le_u64 >> id: cond!(flags.has_id(), le_u64) >> (value, id)
    )
}

pub fn parse_read_format(input: &[u8], flags: ReadFormatFlags) -> IResult<&[u8], ReadFormat> {
    if flags.has_group() {
        do_parse!(
            input,
            nr: le_u64
                >> time_enabled: cond!(flags.has_total_time_enabled(), le_u64)
                >> time_running: cond!(flags.has_total_time_running(), le_u64)
                >> values: count!(call!(parse_read_value, flags), nr as usize)
                >> (ReadFormat {
                    time_enabled: time_enabled,
                    time_running: time_running,
                    values: values
                })
        )
    } else {
        do_parse!(
            input,
            value: le_u64
                >> time_enabled: cond!(flags.has_total_time_enabled(), le_u64)
                >> time_running: cond!(flags.has_total_time_running(), le_u64)
                >> id: cond!(flags.has_id(), le_u64)
                >> (ReadFormat {
                    time_enabled: time_enabled,
                    time_running: time_running,
                    values: vec![(value, id)]
                })
        )
    }
}

named!(pub parse_branch_entry<&[u8], BranchEntry>,
    do_parse!(
        from: le_u64 >>
        to: le_u64 >>
        flags: le_u64 >>
        (BranchEntry {
            from: from,
            to: to,
            flags: flags,
        })
    )
);

pub fn parse_branch_entries(
    input: &[u8],
    flags: SampleFormatFlags,
) -> IResult<&[u8], Vec<BranchEntry>> {
    // TODO: bug? https://github.com/Geal/nom/issues/302
    assert!(flags.has_branch_stack() && flags.has_regs_user());
    do_parse!(
        input,
        // TODO: bug? https://github.com/Geal/nom/issues/302
        //bnr: cond!(flags.has_branch_stack(), le_u64) ~
        //entries: cond!(flags.has_branch_stack() && flags.has_regs_user(), count!(parse_branch_entry, 3)),
        bnr: le_u64 >> entries: count!(parse_branch_entry, bnr as usize) >> (entries)
    )
}

pub fn parse_sample_record<'a>(
    input: &'a [u8],
    attr: &'a EventAttr,
) -> IResult<&'a [u8], SampleRecord> {
    let flags = attr.sample_type;
    let regcnt_user = attr.sample_regs_user.count_ones() as usize;
    let regcnt_intr = attr.sample_regs_intr.count_ones() as usize;
    do_parse!(
        input,
        sample_id: cond!(flags.has_identifier(), le_u64)
            >> ip: cond!(flags.has_ip(), le_u64)
            >> ptid: cond!(flags.has_tid(), parse_thread_id)
            >> time: cond!(flags.has_time(), le_u64)
            >> addr: cond!(flags.has_addr(), le_u64)
            >> id: cond!(flags.has_sample_id(), le_u64)
            >> stream_id: cond!(flags.has_stream_id(), le_u64)
            >> cpu: cond!(flags.has_cpu(), parse_cpu)
            >> period: cond!(flags.has_period(), le_u64)
            >> v: cond!(flags.has_read(), call!(parse_read_format, attr.read_format))
            >> ips: cond!(flags.has_callchain(), parse_vec_u64)
            >> raw: cond!(flags.has_raw(), parse_vec_u32_u8)
            >> lbr: cond!(flags.has_branch_stack(), call!(parse_branch_entries, flags))
            >> abi_user: cond!(flags.has_stack_user(), le_u64)
            >> regs_user:
                cond!(
                    flags.has_stack_user(),
                    call!(parse_vec_u64_variable, regcnt_user)
                )
            >> user_stack_len: cond!(flags.has_stack_user(), le_u64)
            >> user_stack:
                cond!(
                    flags.has_stack_user(),
                    call!(parse_vec_u8_variable, user_stack_len.unwrap() as usize)
                )
            >> dyn_size:
                cond!(
                    flags.has_stack_user() && user_stack_len.unwrap() != 0,
                    le_u64
                )
            >> weight: cond!(flags.has_weight(), le_u64)
            >> data_src: cond!(flags.has_data_src(), le_u64)
            >> transaction: cond!(flags.has_transaction(), le_u64)
            >> abi: cond!(flags.has_regs_intr(), le_u64)
            >> regs_intr:
                cond!(
                    flags.has_regs_intr(),
                    call!(parse_vec_u64_variable, regcnt_intr)
                )
            >> (SampleRecord {
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

pub fn parse_comm_record(input: &[u8]) -> IResult<&[u8], CommRecord> {
    do_parse!(
        input,
        ptid: parse_thread_id >>
        comm: parse_c_string >>
        // TODO: sample_id: parse_sample_id,
        (CommRecord {
            ptid: ptid,
            comm: unsafe { String::from_utf8_unchecked(comm.to_vec()) }
        })
    )
}

/// Parse an event record.
pub fn parse_event<'a>(input: &'a [u8], attrs: &'a Vec<EventAttr>) -> IResult<&'a [u8], Event> {
    do_parse!(
        input,
        header: parse_event_header
            >> event:
                alt!(
                    cond_reduce!(
                        header.event_type == EventType::Mmap,
                        map!(parse_mmap_record, EventData::MMAP)
                    ) | cond_reduce!(
                        header.event_type == EventType::Mmap2,
                        map!(parse_mmap2_record, EventData::MMAP2)
                    ) | cond_reduce!(
                        header.event_type == EventType::Comm,
                        map!(parse_comm_record, EventData::Comm)
                    ) | cond_reduce!(
                        header.event_type == EventType::Exit,
                        map!(parse_exit_record, EventData::Exit)
                    ) | cond_reduce!(
                        header.event_type == EventType::Sample,
                        map!(call!(parse_sample_record, &attrs[0]), EventData::Sample)
                    ) | cond_reduce!(
                        header.event_type == EventType::Fork,
                        map!(parse_fork_record, EventData::Fork)
                    ) | cond_reduce!(
                        header.event_type == EventType::Unthrottle,
                        map!(parse_unthrottle_record, EventData::Unthrottle)
                    ) | cond_reduce!(
                        header.event_type == EventType::Throttle,
                        map!(parse_throttle_record, EventData::Throttle)
                    ) | cond_reduce!(
                        header.event_type == EventType::BuildId,
                        map!(
                            call!(parse_build_id_record, header.size()),
                            EventData::BuildId
                        )
                    ) | cond_reduce!(header.event_type == EventType::FinishedRound, no_event)
                        | cond_reduce!(header.event_type.is_unknown(), no_event)
                )
            >> (Event {
                header: header,
                data: event
            })
    )
}

// Parse a perf file section.
named!(pub parse_file_section<&[u8], PerfFileSection>,
    do_parse!(
        offset: le_u64 >>
        size: le_u64 >>
        (PerfFileSection { offset: offset, size: size })
    )
);

// Parse a perf string.
named!(pub parse_perf_string<&[u8], String>,
    do_parse!(
        length: le_u32 >>
        bytes: take!(length as usize) >>
        ({
            bytes.split(|c| *c == 0x0).next().map(|slice|
                unsafe { String::from_utf8_unchecked(slice.to_vec()) }
            ).unwrap_or(String::new())
        })
    )
);

// Parse a perf string list.
named!(pub parse_perf_string_list<&[u8], Vec<String> >,
    do_parse!(
        nr: le_u32 >>
        strings: count!(parse_perf_string, nr as usize) >>
        (strings)
    )
);

named!(pub parse_nrcpus<&[u8], NrCpus>,
    do_parse!(
        nr_online: le_u32 >>
        nr_available: le_u32 >>
        (NrCpus { online: nr_online, available: nr_available })
    )
);

pub fn parse_event_desc(input: &[u8]) -> IResult<&[u8], Vec<EventDesc>> {
    do_parse!(
        input,
        nr: le_u32
            >> attr_size: le_u32
            >> descs:
                count!(
                    do_parse!(
                        attr: flat_map!(take!(attr_size as usize), parse_event_attr)
                            >> nr_ids: le_u32
                            >> event_string: parse_perf_string
                            >> ids: call!(parse_vec_u64_variable, nr_ids as usize)
                            >> (EventDesc {
                                attr: attr,
                                event_string: event_string,
                                ids: ids
                            })
                    ),
                    nr as usize
                )
            >> (descs)
    )
}

named!(pub parse_cpu_topology<&[u8], CpuTopology>,
    do_parse!(
        cores: parse_perf_string_list >>
        threads: parse_perf_string_list >>
        (CpuTopology { cores: cores, threads: threads })
    )
);

named!(pub parse_numa_node<&[u8], NumaNode>,
    do_parse!(
        nr: le_u32 >>
        mem_total: le_u64 >>
        mem_free: le_u64 >>
        cpu: parse_perf_string >>
        (NumaNode { node_nr: nr, mem_free: mem_free, mem_total: mem_total, cpus: cpu })
    )
);

named!(pub parse_numa_topology<&[u8], Vec<NumaNode> >,
    do_parse!(
        nr: le_u32 >>
        nodes: count!(parse_numa_node, nr as usize) >>
        (nodes)
    )
);

named!(pub parse_pmu_mapping<&[u8], PmuMapping>,
    do_parse!(
        pmu_type: le_u32 >>
        pmu_name: parse_perf_string >>
        (PmuMapping { pmu_name: pmu_name, pmu_type: pmu_type })
    )
);

named!(pub parse_pmu_mappings<&[u8], Vec<PmuMapping> >,
    do_parse!(
        nr: le_u32 >>
        nodes: count!(parse_pmu_mapping, nr as usize) >>
        (nodes)
    )
);

named!(pub parse_group_description<&[u8], GroupDesc>,
    do_parse!(
        string: parse_perf_string >>
        leader_idx: le_u32 >>
        nr_members: le_u32 >>
        (GroupDesc { string: string, leader_idx: leader_idx, nr_members: nr_members })
    )
);

named!(pub parse_group_descriptions<&[u8], Vec<GroupDesc> >,
    do_parse!(
        nr: le_u32 >>
        nodes: count!(parse_group_description, nr as usize) >>
        (nodes)
    )
);

pub fn parse_build_id_record<'a>(
    input: &'a [u8],
    record_size: usize,
) -> IResult<&'a [u8], BuildIdRecord> {
    do_parse!(
        input,
        pid: le_i32 >>
        build_id: take!(24) >>
        filename: take!(record_size - 4 - 24) >> // header.size - offsetof(struct build_id_event, filename)
        (BuildIdRecord {
            pid: pid,
            build_id: build_id.to_owned(),
            filename: unsafe { String::from_utf8_unchecked(filename.to_vec()) }
        })
    )
}

// Parse a perf header
named!(pub parse_header<&[u8], PerfFileHeader>,
    do_parse!(
        tag!("PERFILE2") >>
        size: le_u64 >>
        attr_size: le_u64 >>
        attrs: parse_file_section >>
        data: parse_file_section >>
        event_types: parse_file_section >>
        flags: bits!(do_parse!(
            nrcpus: take_bits!(u8, 1) >>
            arch: take_bits!(u8, 1) >>
            version: take_bits!(u8, 1) >>
            osrelease: take_bits!(u8, 1) >>
            hostname: take_bits!(u8, 1) >>
            build_id: take_bits!(u8, 1) >>
            tracing_data: take_bits!(u8, 1) >>
            take_bits!(u8, 1) >>

            branch_stack: take_bits!(u8, 1) >>
            numa_topology: take_bits!(u8, 1) >>
            cpu_topology: take_bits!(u8, 1) >>
            event_desc: take_bits!(u8, 1) >>
            cmdline: take_bits!(u8, 1) >>
            total_mem: take_bits!(u8, 1) >>
            cpuid: take_bits!(u8, 1) >>
            cpudesc: take_bits!(u8, 1) >>

            take_bits!(u8, 6) >> // padding
            group_desc: take_bits!(u8, 1) >>
            pmu_mappings: take_bits!(u8, 1) >>
            ({
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
            })
        )) >>
        take!(29) >> // reserved
        (PerfFileHeader { size: size, attr_size: attr_size, attrs: attrs, data: data, event_types: event_types, flags: flags })
    )
);

// Parse a perf header
named!(pub parse_event_attr<&[u8], EventAttr>,
    do_parse!(
        attr_type: le_u32 >>
        size: le_u32 >>
        config: le_u64 >>
        sample_period_freq: le_u64 >>
        sample_type: le_u64 >>
        read_format: le_u64 >>
        settings: le_u64 >>
        wakeup_events_watermark: le_u32 >>
        bp_type: le_u32 >>
        config1_or_bp_addr: le_u64 >>
        config2_or_bp_len: le_u64 >>
        branch_sample_type: le_u64 >>
        sample_regs_user: le_u64 >>
        sample_stack_user: le_u32 >>
        clock_id: le_i32 >>
        sample_regs_intr: le_u64 >>
        aux_watermark: le_u32 >>
        le_u32 >> // reserved
        (EventAttr {
            attr_type: attr_type,
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
            reserved: 0
        })
));
