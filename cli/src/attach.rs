use libdrbug::prelude::*;

use crate::repl;

#[derive(clap::Args)]
pub struct Args {
    #[arg(short, long, help = "PID of process to attach to")]
    pid: i32,
}

pub fn cmd(args: &Args) -> Empty {
    println!("attaching to pid {}", args.pid);
    let proc = Process::attach(args.pid)?;

    repl::start(proc)
}
