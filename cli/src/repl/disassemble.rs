use clap::Args;
use iced_x86::{
    Formatter,
    GasFormatter,
};
use libdrbug::prelude::*;

use crate::Empty;

#[derive(Args)]
pub(super) struct DisassembleArgs {
    #[arg(short, long_help = "address to start disassembly")]
    pub(super) addr: Option<VirtAddr>,

    #[arg(
        short = 'n',
        default_value = "5",
        long_help = "number of instructions to disassemble"
    )]
    pub(super) instr_count: usize,
}

pub(super) fn print_disassembly(proc: &mut Process, addr: Option<VirtAddr>, instr_count: usize) -> Empty {
    let dis = Disassembler::new(proc);
    let instructions = dis.disassemble(addr, instr_count)?;

    let mut formatter = GasFormatter::new();
    formatter.options_mut().set_uppercase_hex(false);
    let mut output = String::new();

    for instr in &instructions {
        output.clear();
        formatter.format(instr, &mut output);

        println!("{:#018x}: {output}", instr.ip());
    }

    Ok(())
}
