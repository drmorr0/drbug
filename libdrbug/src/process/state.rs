use std::fmt;

use nix::sys::signal::Signal;
use nix::sys::wait::WaitStatus;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessState {
    Exited { exit_code: i32 },
    Running,
    Stopped { signal: Option<Signal> },
    Terminated { signal: Signal },
    Unknown(WaitStatus),
}

impl ProcessState {
    pub fn is_exited(&self) -> bool {
        matches!(self, ProcessState::Exited { .. })
    }

    pub fn is_running(&self) -> bool {
        *self == ProcessState::Running
    }

    pub fn is_stopped(&self) -> bool {
        matches!(self, ProcessState::Stopped { .. })
    }

    pub fn is_terminated(&self) -> bool {
        matches!(self, ProcessState::Terminated { .. })
    }
}

impl From<WaitStatus> for ProcessState {
    fn from(ws: WaitStatus) -> Self {
        match ws {
            WaitStatus::Exited(_, code) => ProcessState::Exited { exit_code: code },
            WaitStatus::Signaled(_, signal, _) => ProcessState::Terminated { signal },
            WaitStatus::Stopped(_, signal) => ProcessState::Stopped { signal: Some(signal) },
            _ => ProcessState::Unknown(ws),
        }
    }
}

impl fmt::Display for ProcessState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessState::Exited { exit_code } => write!(f, "exited with code {exit_code}"),
            ProcessState::Running => write!(f, "running"),
            ProcessState::Stopped { signal: maybe_signal } => {
                if let Some(signal) = maybe_signal {
                    write!(f, "paused by signal {signal}")
                } else {
                    write!(f, "paused")
                }
            },
            ProcessState::Terminated { signal } => write!(f, "terminated with signal {signal}"),
            ProcessState::Unknown(ws) => write!(f, "unknown: wait status = {ws:?}"),
        }
    }
}
