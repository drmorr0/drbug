use nix::sys::signal::{
    Signal,
    kill,
};
use nix::unistd::Pid;

use super::*;
use crate::process::ProcessOptions;
use crate::{
    DrbugError,
    Empty,
    syscall_error,
};

fn process_exists(pid: Pid) -> Empty {
    // In the book we send signal 0 to kill, which does nothing, but `nix::kill` doesn't support
    // this (there is no enum Signal variant for 0), so instead we send SIGUSR1 which isn't
    // technically correct (since default behaviour is to kill the process), but it works well enough,
    // since if it returns a success that means there was a process to kill.
    syscall_error!(kill(pid, Signal::SIGUSR1))
}

fn get_process_status_char(pid: Pid) -> char {
    // /proc/pid/stat format is
    // pid (executable name) state bunch of random crap
    let procfile = format!("/proc/{pid}/stat");
    let data = std::fs::read_to_string(procfile).unwrap();
    // We have to use rfind to protect against the case where someone named their file `deez )`
    let index_of_last_paren = data.rfind(')').unwrap();
    data.chars().nth(index_of_last_paren + 2usize).unwrap() // skip the paren and the following space
}

#[rstest]
fn test_attach_success() -> Empty {
    let target = Process::launch(LOOP_PATH, ProcessOptions { start_unattached: true, ..Default::default() }).unwrap();
    // if the result from Process::attach is dropped, then it auto-ptrace-detaches, which starts
    // the child process up again, which is _SUPER_ annoying, so we have to make sure it doesn't
    // drop until after we do our assertion
    let _proc = Process::attach(target.pid().into())?;
    assert_eq!(get_process_status_char(target.pid()), 't'); // 't' means paused for tracing
    Ok(())
}

#[rstest]
fn test_attach_invalid_pid() {
    assert_matches!(Process::attach(0), Err(DrbugError::SyscallFailed(..)));
}

#[rstest]
fn test_launch_success() -> Empty {
    let proc = Process::launch(LOOP_PATH, Default::default())?;
    process_exists(proc.pid())
}

#[rstest]
fn test_launch_no_such_program() {
    assert_matches!(Process::launch("deez", Default::default()), Err(DrbugError::ChildProcessFailed(..)));
}

#[rstest]
fn test_resume_success_launch() -> Empty {
    let mut proc = Process::launch(LOOP_PATH, Default::default())?;
    proc.resume()?;
    let status_char = get_process_status_char(proc.pid());
    assert_contains!(vec!['R', 'S'], &status_char); // R = running, S = sleeping
    Ok(())
}

#[rstest]
fn test_resume_success_attach() -> Empty {
    let target = Process::launch(LOOP_PATH, ProcessOptions { start_unattached: true, ..Default::default() })?;
    let mut proc = Process::attach(target.pid().into())?;
    proc.resume()?;
    let status_char = get_process_status_char(proc.pid());
    assert_contains!(vec!['R', 'S'], &status_char); // R = running, S = sleeping
    Ok(())
}

#[rstest]
fn test_resume_process_terminated() -> Empty {
    let mut proc = Process::launch("true", Default::default())?;
    proc.resume()?;
    proc.wait_on_signal()?;
    assert_err!(proc.resume());

    Ok(())
}
