enum PebsType {
    RegularEvent,
    PebsOrRegularEvent,
    PebsOnlyEvent
}

enum EventCode {
    OneCode(u8),
    TwoCodes(u8,u8)
}

enum Counter {
    /// Bit-mask containing the fixed counters
    /// usable with the corresponding performance event.
    Fixed(u8),

    /// Bit-mask containing the programmable counters
    /// usable with the corresponding performance event.
    Programmable(u8),
}

struct IntelPerformanceCounterDescription {

    /// This field maps to the Event Select field in the IA32_PERFEVTSELx[7:0]MSRs.
    ///
    /// The set of values for this field is defined architecturally.
    /// Each value corresponds to an event logic unit and should be used with a unit
    /// mask value to obtain an architectural performance event.
    event_code: EventCode,

    /// This field maps to the Unit Mask filed in the IA32_PERFEVTSELx[15:8] MSRs.
    ///
    /// It further qualifies the event logic unit selected in the event select
    /// field to detect a specific micro-architectural condition.
    umask: u8,

    /// It is a string of characters to identify the programming of an event.
    event_name: &'static str,

    /// This field contains a description of what is being counted by a particular event.
    brief_description: &'static str,

    /// In some cases, this field will contain a more detailed description of what is counted by an event.
    public_description: Option<&'static str>,

    /// This field lists the fixed (PERF_FIXED_CTRX) or programmable (IA32_PMCX)
    /// counters that can be used to count the event.
    counter: Counter,

    /// This field lists the counters where this event can be sampled
    /// when Intel® Hyper-Threading Technology (Intel® HT Technology) is
    /// disabled.
    ///
    /// When Intel® HT Technology is disabled, some processor cores gain access to
    /// the programmable counters of the second thread, making a total of eight
    /// programmable counters available. The additional counters will be
    /// numbered 4,5,6,7. Fixed counter behavior remains unaffected.
    counter_ht_off: Counter,

    /// This field is only relevant to PEBS events.
    ///
    /// It lists the counters where the event can be sampled when it is programmed as a PEBS event.
    pebs_counters: Option<Counter>,

    /// Sample After Value (SAV) is the value that can be preloaded
    /// into the counter registers to set the point at which they will overflow.
    ///
    /// To make the counter overflow after N occurrences of the event,
    /// it should be loaded with (0xFF..FF – N) or –(N-1). On overflow a
    /// hardware interrupt is generated through the Local APIC and additional
    /// architectural state can be collected in the interrupt handler.
    /// This is useful in event-based sampling. This field gives a recommended
    /// default overflow value, which may be adjusted based on workload or tool preference.
    sample_after_value: u64,

    /// Additional MSRs may be required for programming certain events.
    /// This field gives the address of such MSRS.
    msr_index: Option<u64>,

    /// When an MSRIndex is used (indicated by the MSRIndex column), this field will
    /// contain the value that needs to be loaded into the
    /// register whose address is given in MSRIndex column.
    ///
    /// For example, in the case of the load latency events, MSRValue defines the
    /// latency threshold value to write into the MSR defined in MSRIndex (0x3F6).
    msr_value: Option<u64>,

    /// This field is set for an event which can only be sampled or counted by itself,
    /// meaning that when this event is being collected,
    /// the remaining programmable counters are not available to count any other events.
    taken_alone: bool,

    /// This field maps to the Counter Mask (CMASK) field in IA32_PERFEVTSELx[31:24] MSR.
    counter_mask: u8,

    /// This field corresponds to the Invert Counter Mask (INV) field in IA32_PERFEVTSELx[23] MSR.
    invert: bool,

    /// This field corresponds to the Any Thread (ANY) bit of IA32_PERFEVTSELx[21] MSR.
    any_thread: bool,

    /// This field corresponds to the Edge Detect (E) bit of IA32_PERFEVTSELx[18] MSR.
    edge_detect: bool,

    /// A '0' in this field means that the event cannot be programmed as a PEBS event.
    /// A '1' in this field means that the event is a  precise event and can be programmed
    /// in one of two ways – as a regular event or as a PEBS event.
    /// And a '2' in this field means that the event can only be programmed as a PEBS event.
    pebs: PebsType,

    /// A '1' in this field means the event uses the Precise Store feature and Bit 3 and
    /// bit 63 in IA32_PEBS_ENABLE MSR must be set to enable IA32_PMC3 as a PEBS counter
    /// and enable the precise store facility respectively.
    ///
    /// Processors based on SandyBridge and IvyBridge micro-architecture offer a
    /// precise store capability that provides a means to profile store memory
    /// references in the system.
    precise_store: bool,

    /// A '1' in this field means that when the event is configured as a PEBS event,
    /// the Data Linear Address facility is supported.
    ///
    /// The Data Linear Address facility is a new feature added to Haswell as a
    /// replacement or extension of the precise store facility in SNB.
    data_la: bool,

    /// A '1' in this field means that when the event is configured as a PEBS event,
    /// the DCU hit field of the PEBS record is set to 1 when the store hits in the
    /// L1 cache and 0 when it misses.
    l1_hit_indication: bool,

    /// This field lists the known bugs that apply to the events.
    ///
    /// For the latest, up to date errata refer to the following links:
    ////
    /// * Haswell:
    ///   http://www.intel.com/content/dam/www/public/us/en/documents/specification-updates/4th-gen-core-family-mobile-specification-update.pdf
    ///
    /// * IvyBridge:
    ///   https://www-ssl.intel.com/content/dam/www/public/us/en/documents/specification-updates/3rd-gen-core-desktop-specification-update.pdf
    ///
    /// * SandyBridge:
    ///   https://www-ssl.intel.com/content/dam/www/public/us/en/documents/specification-updates/2nd-gen-core-family-mobile-specification-update.pdf
    errata: Option<&'static str>,

    /// There is only 1 file for core and offcore events in this format.
    /// This field is set to 1 for offcore events and 0 for core events.
    offcore: bool,
}
