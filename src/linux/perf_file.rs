//! High-level abstractions for a `perf.data` file.
//!
//! # References
//! more informations about the perf.data format can be found here:
//!
//!  * https://lwn.net/Articles/644919/
//!  * http://man7.org/linux/man-pages/man2/perf_event_open.2.html
//!  * https://github.com/andikleen/pmu-tools/tree/master/parser
//!

use super::parser::*;
use super::perf_format::*;
use nom::*;

macro_rules! stderr {
    ($($arg:tt)*) => (
        use std::io::Write;
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr (file handle closed?): {}", x),
        }
    )
}

fn iresult_to_option<I, O, E>(result: IResult<I, O, E>) -> Option<O> {
    match result {
        Ok((_, res)) => Some(res),
        Err(_) => None,
    }
}

#[derive(Debug)]
pub struct PerfFile {
    pub header: PerfFileHeader,
    pub attrs: Vec<EventAttr>,
    bytes: Vec<u8>,
    //sections: Vec<PerfFileSection>,
}

pub struct PerfFileEventDataIter<'a> {
    attrs: &'a Vec<EventAttr>,
    data: &'a [u8],
    offset: usize,
}

impl<'a> Iterator for PerfFileEventDataIter<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = &self.data[self.offset..];
        if slice.len() > 8 {
            let r = parse_event(slice, self.attrs);
            match r {
                Ok((_, ev)) => {
                    self.offset += ev.header.size();
                    Some(ev)
                }
                Err(nom::Err::Error(_)) | Err(nom::Err::Failure(_)) => {
                    stderr!("Error when parsing data section.");
                    None
                }
                Err(nom::Err::Incomplete(n)) => {
                    stderr!("Got incomplete data ({:?}) when parsing data section.", n);
                    None
                }
            }
        } else {
            None
        }
    }
}

impl PerfFile {
    pub fn new(bytes: Vec<u8>) -> PerfFile {
        let header = match parse_header(bytes.as_slice()) {
            Ok((_, h)) => h,
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => panic!("{:?}", e),
            Err(nom::Err::Incomplete(_)) => panic!("Incomplete data?"),
        };

        let attrs = {
            let attr_size = header.attr_size as usize;
            let slice: &[u8] = &bytes[header.attrs.start()..header.attrs.end()];
            slice
                .chunks(attr_size)
                .map(|c| parse_event_attr(c).unwrap().1)
                .collect()
        };

        PerfFile {
            bytes: bytes,
            header: header,
            attrs: attrs,
        }
    }

    pub fn data(&self) -> PerfFileEventDataIter {
        let slice: &[u8] = &self.bytes[self.header.data.start()..self.header.data.end()];
        PerfFileEventDataIter {
            attrs: &self.attrs,
            data: slice,
            offset: 0,
        }
    }

    pub fn get_build_id(&self) -> Option<BuildIdRecord> {
        self.get_section_slice(HeaderFlag::BuildId)
            .and_then(|slice| {
                iresult_to_option(do_parse!(
                    slice,
                    header: parse_event_header
                        >> build_id: call!(parse_build_id_record, header.size())
                        >> (build_id)
                ))
            })
    }

    pub fn get_hostname(&self) -> Option<String> {
        self.get_section_slice(HeaderFlag::Hostname)
            .and_then(|slice| iresult_to_option(parse_perf_string(slice)))
    }

    pub fn get_os_release(&self) -> Option<String> {
        self.get_section_slice(HeaderFlag::OsRelease)
            .and_then(|slice| iresult_to_option(parse_perf_string(slice)))
    }

    pub fn get_version(&self) -> Option<String> {
        self.get_section_slice(HeaderFlag::Version)
            .and_then(|slice| iresult_to_option(parse_perf_string(slice)))
    }

    pub fn get_arch(&self) -> Option<String> {
        self.get_section_slice(HeaderFlag::Arch)
            .and_then(|slice| iresult_to_option(parse_perf_string(slice)))
    }

    pub fn get_nr_cpus(&self) -> Option<NrCpus> {
        self.get_section_slice(HeaderFlag::NrCpus)
            .and_then(|slice| iresult_to_option(parse_nrcpus(slice)))
    }

    pub fn get_cpu_description(&self) -> Option<String> {
        self.get_section_slice(HeaderFlag::CpuDesc)
            .and_then(|slice| iresult_to_option(parse_perf_string(slice)))
    }

    pub fn get_cpu_id(&self) -> Option<String> {
        self.get_section_slice(HeaderFlag::CpuId)
            .and_then(|slice| iresult_to_option(parse_perf_string(slice)))
    }

    pub fn get_total_memory(&self) -> Option<u64> {
        self.get_section_slice(HeaderFlag::TotalMem)
            .and_then(|slice| iresult_to_option(le_u64(slice)))
    }

    pub fn get_cmd_line(&self) -> Option<String> {
        self.get_section_slice(HeaderFlag::CmdLine)
            .and_then(|slice| iresult_to_option(parse_perf_string(slice)))
    }

    pub fn get_event_description(&self) -> Option<Vec<EventDesc>> {
        self.get_section_slice(HeaderFlag::EventDesc)
            .and_then(|slice| iresult_to_option(parse_event_desc(slice)))
    }

    pub fn get_cpu_topology(&self) -> Option<CpuTopology> {
        self.get_section_slice(HeaderFlag::CpuTopology)
            .and_then(|slice| iresult_to_option(parse_cpu_topology(slice)))
    }

    pub fn get_numa_topology(&self) -> Option<Vec<NumaNode>> {
        self.get_section_slice(HeaderFlag::NumaTopology)
            .and_then(|slice| iresult_to_option(parse_numa_topology(slice)))
    }

    pub fn get_pmu_mappings(&self) -> Option<Vec<PmuMapping>> {
        self.get_section_slice(HeaderFlag::PmuMappings)
            .and_then(|slice| iresult_to_option(parse_pmu_mappings(slice)))
    }

    pub fn get_group_descriptions(&self) -> Option<Vec<GroupDesc>> {
        self.get_section_slice(HeaderFlag::GroupDesc)
            .and_then(|slice| iresult_to_option(parse_group_descriptions(slice)))
    }

    fn sections(&self) -> Vec<(HeaderFlag, PerfFileSection)> {
        let sections: Vec<PerfFileSection> = self.parse_header_sections().unwrap().1;
        let flags: Vec<HeaderFlag> = self.header.flags.collect();
        assert!(sections.len() == flags.len());

        flags.into_iter().zip(sections).collect()
    }

    fn get_section(&self, sec: HeaderFlag) -> Option<PerfFileSection> {
        let sections = self.sections();
        sections.iter().find(|c| c.0 == sec).map(|c| c.1)
    }

    fn get_section_slice(&self, sec: HeaderFlag) -> Option<&[u8]> {
        self.get_section(sec)
            .map(|sec| &self.bytes[sec.start()..sec.end()])
    }

    fn parse_header_sections(&self) -> IResult<&[u8], Vec<PerfFileSection>> {
        let flags: Vec<HeaderFlag> = self.header.flags.collect();
        // TODO: if flags.len() == 0
        let sections_start: usize = (self.header.data.offset + self.header.data.size) as usize;
        let slice: &[u8] = &self.bytes[sections_start..];

        count!(slice, parse_file_section, flags.len())
    }
}
