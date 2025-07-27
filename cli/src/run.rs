use libdrbug::prelude::*;

use crate::Empty;
use crate::repl::Repl;

#[derive(clap::Args)]
pub struct Args {
    #[arg(help = "path to executable to debug")]
    path: String,
}

pub fn cmd(args: &Args) -> Empty {
    let proc = Process::launch(&args.path, Default::default())?;
    println!("launched process `{}` with PID {}", args.path, proc.pid());

    let mut repl = Repl::new(proc)?;
    repl.start()
}
