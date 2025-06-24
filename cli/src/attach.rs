use libdrbug::prelude::*;
use nix::sys::ptrace;
use nix::sys::wait::waitpid;
use nix::unistd::Pid;

use crate::repl;

#[derive(clap::Args)]
pub struct Args {
    #[arg(short, long, help = "PID of process to attach to")]
    pid: i32,
}

pub fn cmd(args: &Args) -> Empty {
    println!("attaching to pid {}", args.pid);
    let pid = Pid::from_raw(args.pid);
    ptrace::attach(pid)?;
    waitpid(pid, None)?;

    repl::start(pid)
}
