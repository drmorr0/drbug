use clap::{
    Args,
    Subcommand,
};
use libdrbug::prelude::*;

#[derive(Subcommand)]
pub(super) enum RegisterCommand {
    #[command(about = "read from the program registers")]
    Read(RegisterReadArgs),

    #[command(about = "write to the program registers")]
    Write(RegisterWriteArgs),
}

#[derive(Args)]
pub(super) struct RegisterReadArgs {
    regs: Option<String>,
}

#[derive(Args)]
pub(super) struct RegisterWriteArgs {
    reg: String,
    value: String,
}

pub(super) fn handle(command: &RegisterCommand, proc: &mut Process) -> Empty {
    match command {
        RegisterCommand::Read(args) => handle_read(args, proc),
        RegisterCommand::Write(args) => handle_write(args, proc),
    }
}

fn handle_read(args: &RegisterReadArgs, proc: &mut Process) -> Empty {
    let reg_values = match args.regs.as_deref() {
        Some("all") => proc.get_registers().read_group(None)?,
        Some(name) => vec![(name, Some(proc.get_registers().read_by_name(name)?))],
        None => proc.get_registers().read_group(Some(RegisterType::General))?,
    };

    for (reg, val) in reg_values {
        if reg == "orig_rax" {
            continue;
        }
        match val {
            Some(v) => println!("{reg}:\t{v}"),
            None => println!("{reg}:\tnone (unsupported)"),
        }
    }
    Ok(())
}

fn handle_write(_args: &RegisterWriteArgs, _proc: &mut Process) -> Empty {
    Ok(())
}
