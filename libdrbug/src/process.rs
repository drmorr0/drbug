use std::ffi::CString;
use std::ops::Drop;

use nix::sys::ptrace;
use nix::sys::signal::{
    Signal,
    kill,
};
use nix::sys::wait::{
    WaitStatus,
    waitpid,
};
use nix::unistd::{
    ForkResult,
    Pid,
    execvp,
    fork,
};

use crate::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessState {
    Stopped,
    Running,
    Exited,
    Terminated,
}

pub struct Process {
    pid: Pid,
    state: ProcessState,
    terminate_on_end: bool,
}

impl Process {
    pub fn attach(pid_int: i32) -> anyhow::Result<Self> {
        let pid = Pid::from_raw(pid_int);
        ptrace::attach(pid)?;
        Self::wait_new(pid, false)
    }

    pub fn launch(path: String) -> anyhow::Result<Self> {
        let path_cstring = CString::new(path)?;
        let fork_res = unsafe { fork()? };
        let ForkResult::Parent { child } = fork_res else {
            ptrace::traceme()?;
            // First argument is always the name of the program being run
            // TODO: handle additional program arguments here after a -- separator
            execvp(path_cstring.as_c_str(), &[&path_cstring])?;
            unreachable!();
        };

        Self::wait_new(child, true)
    }

    pub fn resume(&mut self) -> Empty {
        ptrace::cont(self.pid, None)?;
        self.state = ProcessState::Running;
        Ok(())
    }

    pub fn wait_on_signal(&self) -> anyhow::Result<WaitStatus> {
        waitpid(self.pid, None).map_err(|e| e.into())
    }

    pub fn pid(&self) -> Pid {
        self.pid
    }

    pub fn state(&self) -> ProcessState {
        self.state
    }

    fn wait_new(pid: Pid, terminate_on_end: bool) -> anyhow::Result<Self> {
        let proc = Process {
            pid,
            terminate_on_end,
            state: ProcessState::Stopped,
        };
        proc.wait_on_signal()?;
        Ok(proc)
    }
}

impl Drop for Process {
    // No error handling; "You'll just have to believe in your destructor"
    fn drop(&mut self) {
        // Must be stopped before you can ptrace::detach
        if self.state == ProcessState::Running {
            let _ = kill(self.pid, Signal::SIGSTOP);
            let _ = waitpid(self.pid, None);
        }

        let _ = ptrace::detach(self.pid, None);
        let _ = kill(self.pid, Signal::SIGCONT);

        if self.terminate_on_end {
            let _ = kill(self.pid, Signal::SIGKILL);
            let _ = waitpid(self.pid, None);
        }
    }
}
