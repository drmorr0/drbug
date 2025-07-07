use libdrbug::prelude::*;

use crate::repl;

#[derive(clap::Args)]
pub struct Args {
    #[arg(help = "PID of process to attach to")]
    path: String,
}

pub fn cmd(args: &Args) -> Empty {
    println!("running program at {}", args.path);
    let proc = Process::launch(&args.path, Default::default())?;
    repl::start(proc)
}
