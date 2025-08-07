use std::cmp::min;

use clap::{
    Args,
    Subcommand,
};
use itertools::Itertools;
use libdrbug::prelude::*;

use crate::Empty;
use crate::parsing::parse_bytes;

#[derive(Subcommand)]
pub(super) enum MemoryCommand {
    #[command(about = "read from program memory", visible_aliases = &["r"])]
    Read(MemReadArgs),

    #[command(about = "write to program memory", visible_aliases = &["w"])]
    Write(MemWriteArgs),
}

#[derive(Args)]
pub(super) struct MemReadArgs {
    #[arg(long_help = "memory address to read")]
    location: VirtAddr,

    #[arg(long_help = "number of bytes to read", default_value = "32")]
    size: usize,
}

#[derive(Args, Clone)]
pub(super) struct MemWriteArgs {
    #[arg(long_help = "memory address to write to")]
    location: VirtAddr,

    #[arg(long_help = "data to write")]
    data_str: String,
}

pub(super) fn handle(command: &MemoryCommand, proc: &mut Process) -> Empty {
    match command {
        MemoryCommand::Read(args) => handle_read(args, proc),
        MemoryCommand::Write(args) => handle_write(args, proc),
    }
}

fn handle_read(args: &MemReadArgs, proc: &mut Process) -> Empty {
    let data = proc.read_memory(args.location, args.size)?;
    for i in (0..data.len()).step_by(16) {
        let end = min(i + 16, data.len());
        println!(
            "{:#016x}: {}",
            args.location.add(i),
            &data[i..end].iter().format_with(" ", |b, f| f(&format_args!("{b:02x}")))
        );
    }
    Ok(())
}

fn handle_write(args: &MemWriteArgs, proc: &mut Process) -> Empty {
    let data = parse_bytes(&args.data_str)?;
    proc.write_memory(args.location, &data)?;
    Ok(())
}
