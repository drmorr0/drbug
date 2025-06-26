use nix::sys::signal::{
    Signal,
    kill,
};
use nix::unistd::Pid;

use super::*;

const LOOP_PATH: &str = "../target/debug/loop";

fn process_exists(pid: Pid) -> Empty {
    // In the book we send signal 0 to kill, which does nothing, but `nix::kill` doesn't support
    // this (there is no enum Signal variant for 0), so instead we send SIGUSR1 which isn't
    // technically correct (since default behaviour is to kill the process), but it works well enough,
    // since if it returns a success that means there was a process to kill.
    kill(pid, Signal::SIGUSR1).map_err(|e| e.into())
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
fn test_attach_success() {
    let target = Process::launch_no_attach(LOOP_PATH).unwrap();
    // if the result from Process::attach is dropped, then it auto-ptrace-detaches, which starts
    // the child process up again, which is _SUPER_ annoying, so we have to make sure it doesn't
    // drop until after we do our assertion
    let _proc = Process::attach(target.pid().into()).unwrap();
    assert_eq!(get_process_status_char(target.pid()), 't'); // 't' means paused for tracing
}

#[rstest]
fn test_attach_invalid_pid() {
    assert_err!(Process::attach(0));
}

#[rstest]
fn test_launch_success() {
    let proc = Process::launch("yes").unwrap();
    process_exists(proc.pid()).unwrap();
}

#[rstest]
fn test_launch_no_such_program() {
    assert_err!(Process::launch("deez"));
}

#[rstest]
fn test_resume_success_launch() {
    let mut proc = Process::launch(LOOP_PATH).unwrap();
    proc.resume().unwrap();
    let status_char = get_process_status_char(proc.pid());
    assert_contains!(vec!['R', 'S'], &status_char); // R = running, S = sleeping
}

#[rstest]
fn test_resume_success_attach() {
    let target = Process::launch_no_attach(LOOP_PATH).unwrap();
    let mut proc = Process::attach(target.pid().into()).unwrap();
    proc.resume().unwrap();
    let status_char = get_process_status_char(proc.pid());
    assert_contains!(vec!['R', 'S'], &status_char); // R = running, S = sleeping
}

#[rstest]
fn test_resume_process_terminated() {
    let mut proc = Process::launch("true").unwrap();
    proc.resume().unwrap();
    proc.wait_on_signal().unwrap();
    assert_err!(proc.resume());
}
