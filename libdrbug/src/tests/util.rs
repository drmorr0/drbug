use std::path::Path;
use std::str;

use elf::ElfBytes;
use elf::endian::AnyEndian;
use nix::unistd::Pid;
use regex::Regex;

use crate::DrbugResult;
use crate::address::VirtAddr;

pub(crate) fn addr_from_bytes(data: &[u8]) -> DrbugResult<VirtAddr> {
    let ptr = u64::from_str_radix(&str::from_utf8(data).unwrap(), 16)?;
    Ok(VirtAddr(ptr))
}

pub(crate) fn get_entry_point_offset(path: &Path) -> u64 {
    let file_data = std::fs::read(path).unwrap();
    let elf_data = ElfBytes::<AnyEndian>::minimal_parse(&file_data).unwrap();

    let entry_file_address = elf_data.ehdr.e_entry;
    let text = elf_data.section_header_by_name(".text").unwrap().unwrap();
    let load_bias = text.sh_addr - text.sh_offset;
    entry_file_address - load_bias
}

pub(crate) fn get_load_addr(pid: Pid, offset: u64) -> VirtAddr {
    let maps = format!("/proc/{pid}/maps");
    let re = Regex::new(r"((?<low_range>\w+)-\w+ ..(?<xbit>.). (?<file_offset>\w+))").unwrap();
    let map_data = std::fs::read(maps).unwrap();
    let map_str = str::from_utf8(&map_data).unwrap();

    for line in map_str.split("\n") {
        if let Some(caps) = re.captures(line)
            && caps["xbit"] == *"x"
        {
            let low_range = u64::from_str_radix(&caps["low_range"], 16).unwrap();
            let file_offset = u64::from_str_radix(&caps["file_offset"], 16).unwrap();

            return VirtAddr(offset - file_offset + low_range);
        }
    }
    panic!("could not find load address");
}
