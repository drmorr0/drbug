use libdrbug::prelude::*;

#[derive(clap::Args)]
pub struct Args {
    #[arg(help = "PID of process to attach to")]
    path: String,
}

pub fn cmd(args: &Args) -> Empty {
    println!("running program at {}", args.path);
    Ok(())
}
