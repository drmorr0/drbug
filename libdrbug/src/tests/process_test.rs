use nix::sys::signal::{
    Signal,
    kill,
};
use nix::unistd::Pid;

use super::*;

fn process_exists(pid: Pid) -> Empty {
    kill(pid, Signal::SIGUSR1).map_err(|e| e.into())
}

#[rstest]
fn test_launch_success() {
    let proc = Process::launch("yes".into()).unwrap();
    process_exists(proc.pid()).unwrap();
}

#[rstest]
fn test_launch_no_such_program() {
    assert_err!(Process::launch("deez".into()));
}
