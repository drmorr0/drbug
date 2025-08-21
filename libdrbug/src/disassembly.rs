use iced_x86::{
    Decoder,
    DecoderOptions,
    Instruction,
};

use crate::DrbugResult;
use crate::address::VirtAddr;
use crate::process::Process;

const BITNESS: u32 = 64;

pub struct Disassembler<'a> {
    proc: &'a mut Process,
}

impl<'a> Disassembler<'a> {
    pub fn new(proc: &'a mut Process) -> Self {
        Disassembler { proc }
    }

    pub fn disassemble(&self, addr: Option<VirtAddr>, instr_count: usize) -> DrbugResult<Vec<Instruction>> {
        let pc = addr.unwrap_or_else(|| self.proc.get_pc().unwrap());
        let code = self.proc.read_memory_without_traps(pc, instr_count * 15)?; // The largest x86 instruction is 15 bytes
        let decoder = Decoder::with_ip(BITNESS, &code, pc.into(), DecoderOptions::NONE);

        Ok(decoder.into_iter().take(instr_count).collect())
    }
}
