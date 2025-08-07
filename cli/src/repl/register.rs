use clap::{
    Args,
    Subcommand,
};
use libdrbug::prelude::*;

use crate::Empty;
use crate::parsing::parse_for_register;

#[derive(Subcommand)]
pub(super) enum RegisterCommand {
    #[command(about = "read from the program registers", visible_aliases = &["r"])]
    Read(RegReadArgs),

    #[command(about = "write to the program registers", visible_aliases = &["w"])]
    Write(RegWriteArgs),
}

#[derive(Args)]
pub(super) struct RegReadArgs {
    #[arg(long_help = "register name to read")]
    regs: Option<String>,
}

#[derive(Args, Clone)]
pub(super) struct RegWriteArgs {
    #[arg(long_help = "register name to write")]
    reg: String,

    #[arg(long_help = "value to write to the register")]
    value: String,
}

pub(super) fn handle(command: &RegisterCommand, proc: &mut Process) -> Empty {
    match command {
        RegisterCommand::Read(args) => handle_read(args, proc),
        RegisterCommand::Write(args) => handle_write(args, proc),
    }
}

fn handle_read(args: &RegReadArgs, proc: &mut Process) -> Empty {
    let reg_values = match args.regs.as_deref() {
        Some("all") => proc.get_registers().read_group(None)?,
        Some(name) => {
            let info = register_info_by_name(name)?;
            vec![(name, Some(proc.get_registers().read(info)?))]
        },
        None => proc.get_registers().read_group(Some(RegisterType::General))?,
    };

    for (reg, value) in reg_values {
        if reg == "orig_rax" {
            continue;
        }
        match value {
            Some(v) => println!("{reg}:\t{v}"),
            None => println!("{reg}:\tnone (unsupported)"),
        }
    }
    Ok(())
}

fn handle_write(args: &RegWriteArgs, proc: &mut Process) -> Empty {
    let info = register_info_by_name(&args.reg)?;
    let value = parse_for_register(info, &args.value)?;
    proc.get_registers_mut().write(info, value)?;
    Ok(())
}
