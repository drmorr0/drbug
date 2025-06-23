mod attach;
mod run;

use clap::{
    Parser,
    Subcommand,
    crate_version,
};
use libdrbug::prelude::*;

#[derive(Parser)]
#[command(about = "x86_64 debugger written in Rust", version, propagate_version = true)]
struct DrbCmd {
    #[command(subcommand)]
    subcommand: DrbSubcommand,
}

#[derive(Subcommand)]
enum DrbSubcommand {
    #[command(about = "attach to a running process")]
    Attach(attach::Args),

    #[command(about = "run the debugger with a specified program")]
    Run(run::Args),

    #[command(about = "drbug version information")]
    Version,
}

fn main() -> Empty {
    let args = DrbCmd::parse();
    match &args.subcommand {
        DrbSubcommand::Attach(args) => attach::cmd(args),
        DrbSubcommand::Run(args) => run::cmd(args),
        DrbSubcommand::Version => {
            println!("drbug {}", crate_version!());
            Ok(())
        },
    }
}
