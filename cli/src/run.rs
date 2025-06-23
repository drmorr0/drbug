use std::ffi::CString;

use libdrbug::prelude::*;
use nix::sys::ptrace;
use nix::sys::wait::waitpid;
use nix::unistd::{
    ForkResult,
    execvp,
    fork,
};

#[derive(clap::Args)]
pub struct Args {
    #[arg(help = "PID of process to attach to")]
    path: String,
}

pub fn cmd(args: &Args) -> Empty {
    println!("running program at {}", args.path);
    let path_cstring = CString::new(args.path.clone())?;

    match unsafe { fork() }? {
        ForkResult::Parent { child } => {
            waitpid(child, None)?;
        },
        ForkResult::Child => {
            ptrace::traceme()?;
            // First argument is always the name of the program being run
            // TODO: handle additional program arguments here after a -- separator
            execvp(path_cstring.as_c_str(), &[&path_cstring])?;
        },
    }
    Ok(())
}
