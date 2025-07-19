use libdrbug::prelude::*;

use crate::Empty;
use crate::repl::Repl;

#[derive(clap::Args)]
pub struct Args {
    #[arg(help = "path to executable to debug")]
    path: String,
}

pub fn cmd(args: &Args) -> Empty {
    println!("running program at {}", args.path);

    let proc = Process::launch(&args.path, Default::default())?;
    let mut repl = Repl::new(proc)?;
    repl.start()
}
