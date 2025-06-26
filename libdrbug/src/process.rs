use std::ffi::CString;
use std::io::Write;
use std::ops::Drop;

use anyhow::bail;
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

#[derive(Debug, Default)]
struct WaitOptions {
    terminate_on_end: bool,
    attached: bool,
}

#[derive(Debug)]
pub struct Process {
    pid: Pid,
    state: ProcessState,
    opts: WaitOptions,
}

impl Process {
    pub fn attach(pid_int: i32) -> anyhow::Result<Self> {
        let pid = Pid::from_raw(pid_int);
        ptrace::attach(pid)?;
        Self::new_then_wait(pid, WaitOptions { terminate_on_end: false, attached: true })
    }

    pub fn launch(path: &str) -> anyhow::Result<Self> {
        Self::launch_maybe_attach(path, true)
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

    fn launch_maybe_attach(path: &str, attach: bool) -> anyhow::Result<Self> {
        let mut channel = Pipe::new_exec_safe()?;

        let path_cstring = CString::new(path)?;
        let fork_res = unsafe { fork()? };
        let ForkResult::Parent { child } = fork_res else {
            // The child process doesn't need the reader, so we close it
            channel.close_reader();

            // in the child process, we can't just use `?`, since it won't get
            // communicated back to the parent; instead we use the channel.
            // Then we still return the error so the child process exits.

            if attach {
                // ptrace::traceme sets up the "attach" in reverse, the child sets up tracing, and
                // then Linux will pause the process on an exec call; this is why we don't need an
                // explicit ptrace::attach call in this case
                if let Err(e) = ptrace::traceme() {
                    let _ = write!(&mut channel, "tracing failed: {e:?}");
                    return Err(e.into());
                }
            }

            // execvp returns Result<Infallible>, i.e., if it ever returns anything
            // it is guaranteed to be an Error, hence the irrefutable_let_patterns
            // warning here; however, I still think it's more clear to use the if let,
            // particularly now that I've written this long comment to justify it.
            //
            // First argument of a program is always the name of the program being run,
            // hence the duplicated path_cstring param.
            //
            // TODO: handle additional program arguments here after a -- separator
            #[allow(irrefutable_let_patterns)]
            if let Err(e) = execvp(path_cstring.as_c_str(), &[&path_cstring]) {
                let _ = write!(&mut channel, "exec failed: {e:?}");
                return Err(e.into());
            }

            // The channel writer is auto-closed here, either because we execvp'ed or because we
            // exited the child process.
            unreachable!();
        };

        // It took me a while to understand why this is necessary, so I'll document it here for
        // posterity.  We have two file descriptors in the pipe, a "read" FD and "write" FD.  We
        // also have two handles to each FD, two in the parent process and two in the child
        // process.  The kernel keeps the pipe open as long as there's an open handle pointing to
        // it: so in the case where everything is successful, the child process will execvp and
        // close both of _its_ handles because of O_CLOEXEC, but the _parent_ process still has
        // the writer handle open, which means the reader will still wait for data.  So we must
        // close the writer in the parent process _before_ we try to read, so that we don't
        // deadlock.
        //
        // In the unhappy path, where the child process fails, it writes data to the pipe and then
        // shuts down, which again closes both of its handles.  However, even if the parent's
        // writer were still open, the read call might return because it got some bytes from the
        // child.  So weirdly, not closing the parent writer works in the unhappy path but it
        // breaks in the happy path.
        //
        // I _think_ closing the reader after we're done is not _strictly_ necessary because they
        // will close when the Process drops but it's (apparently) good practice to close the FDs
        // as soon as they're not needed anymore, so we do it anyways.
        channel.close_writer();
        let data = channel.read()?;
        channel.close_reader();

        if !data.is_empty() {
            waitpid(child, None)?;
            bail!("child process failed with error: {}", std::str::from_utf8(&data)?);
        }

        Self::new_then_wait(child, WaitOptions { terminate_on_end: true, attached: attach })
    }

    fn new_then_wait(pid: Pid, opts: WaitOptions) -> anyhow::Result<Self> {
        let proc = Process { pid, state: ProcessState::Stopped, opts };
        if proc.opts.attached {
            proc.wait_on_signal()?;
        }
        Ok(proc)
    }
}

impl Drop for Process {
    // No error handling; "You'll just have to believe in your destructor"
    fn drop(&mut self) {
        if self.opts.attached {
            // Must be stopped before you can ptrace::detach
            if self.state == ProcessState::Running {
                let _ = kill(self.pid, Signal::SIGSTOP);
                let _ = waitpid(self.pid, None);
            }
        }

        let _ = ptrace::detach(self.pid, None);
        let _ = kill(self.pid, Signal::SIGCONT);

        if self.opts.terminate_on_end {
            let _ = kill(self.pid, Signal::SIGKILL);
            let _ = waitpid(self.pid, None);
        }
    }
}

#[cfg(test)]
impl Process {
    pub(crate) fn launch_no_attach(path: &str) -> anyhow::Result<Self> {
        Self::launch_maybe_attach(path, false)
    }
}
