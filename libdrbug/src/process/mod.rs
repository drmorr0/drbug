mod state;

use std::ffi::CString;
use std::io::Write;
use std::ops::Drop;
use std::os::fd::OwnedFd;

use nix::sys::ptrace;
use nix::sys::signal::{
    Signal,
    kill,
};
use nix::sys::wait::waitpid;
use nix::unistd::{
    ForkResult,
    Pid,
    dup2_stdout,
    execvp,
    fork,
};

use self::state::ProcessState;
use crate::address::VirtAddr;
use crate::breakpoint::{
    BreakList,
    Breakable,
    BreakpointSite,
};
use crate::pipe::Pipe;
use crate::register::Registers;
use crate::register::info::{
    RegisterId,
    register_info_by_id,
};
use crate::register::value::RegisterValue;
use crate::{
    DrbugError,
    DrbugResult,
    Empty,
    ptrace_error,
    syscall_error,
};

#[derive(Default)]
pub struct ProcessOptions {
    pub start_unattached: bool, // use the negative here so the default does the right thing
    pub stdout: Option<OwnedFd>,
}

#[derive(Debug)]
pub struct Process {
    attached: bool,
    breakpoint_sites: BreakList<BreakpointSite>,
    pid: Pid,
    registers: Registers,
    state: ProcessState,
    terminate_on_end: bool,
}

impl Process {
    fn new_then_wait(pid: Pid, opts: ProcessOptions, terminate_on_end: bool) -> DrbugResult<Self> {
        let mut proc = Process {
            attached: !opts.start_unattached,
            breakpoint_sites: BreakList::new(),
            pid,
            registers: Registers::new(pid),
            state: ProcessState::Stopped { signal: None },
            terminate_on_end,
        };
        if proc.attached {
            proc.wait_on_signal()?;
        }
        Ok(proc)
    }

    pub fn attach(pid_int: i32) -> DrbugResult<Self> {
        let pid = Pid::from_raw(pid_int);
        ptrace_error!(attach(pid))?;
        Self::new_then_wait(pid, Default::default(), false)
    }

    pub fn launch(path: &str, opts: ProcessOptions) -> DrbugResult<Self> {
        let mut channel = Pipe::new_exec_safe()?;

        let path_cstring = CString::new(path)?;
        let fork_res = unsafe { syscall_error!(fork())? };
        let ForkResult::Parent { child } = fork_res else {
            // in the child process, we can't just use `?`, since it won't get
            // communicated back to the parent; instead we use the channel.
            // Then we still return the error so the child process exits.

            // The child process doesn't need the reader, so we close it
            channel.close_reader();

            // Replace stdout of the child process so our debugger and/or test harness can read it
            if let Some(fd) = opts.stdout {
                if let Err(e) = dup2_stdout(fd) {
                    let _ = write!(&mut channel, "dup2_stdout failed: {e:?}");
                    return Err(DrbugError::SyscallFailed("dup2", e));
                }
            }

            if !opts.start_unattached {
                // ptrace::traceme sets up the "attach" in reverse, the child sets up tracing, and
                // then Linux will pause the process on an exec call; this is why we don't need an
                // explicit ptrace::attach call in this case
                if let Err(e) = ptrace::traceme() {
                    let _ = write!(&mut channel, "tracing failed: {e:?}");
                    return Err(DrbugError::SyscallFailed("ptrace", e));
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
                return Err(DrbugError::SyscallFailed("execvp", e));
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
            syscall_error!(waitpid(child, None))?;
            return Err(DrbugError::ChildProcessFailed(String::from_utf8_lossy(&data).into()));
        }

        Self::new_then_wait(child, opts, true)
    }

    pub fn breakpoint_sites(&self) -> &BreakList<BreakpointSite> {
        &self.breakpoint_sites
    }

    pub fn breakpoint_sites_mut(&mut self) -> &mut BreakList<BreakpointSite> {
        &mut self.breakpoint_sites
    }

    pub fn create_breakpoint_site(&mut self, addr: VirtAddr) -> DrbugResult<BreakpointSite> {
        if let Some(site) = self.breakpoint_sites.get_by_addr(&addr) {
            return Err(DrbugError::BreakpointSiteExists(site.id(), addr));
        }

        let site = BreakpointSite::new(addr);
        self.breakpoint_sites.add(site.clone());
        Ok(site)
    }

    pub fn get_pc(&self) -> DrbugResult<VirtAddr> {
        let rip_info = register_info_by_id(&RegisterId::rip);
        self.registers.read(rip_info).map(|v| match v {
            RegisterValue::U64(rip) => VirtAddr(rip),
            _ => panic!("should never happen"),
        })
    }

    pub fn resume(&mut self) -> Empty {
        ptrace_error!(cont(self.pid, None))?;
        self.state = ProcessState::Running;
        Ok(())
    }

    pub fn wait_on_signal(&mut self) -> DrbugResult<ProcessState> {
        let res = syscall_error!(waitpid(self.pid, None))?;
        self.state = res.into();

        if self.attached && self.state.is_stopped() {
            self.registers.load_all()?;
        }
        Ok(self.state)
    }

    pub fn get_registers(&self) -> &Registers {
        &self.registers
    }

    pub fn get_registers_mut(&mut self) -> &mut Registers {
        &mut self.registers
    }

    pub fn pid(&self) -> Pid {
        self.pid
    }

    pub fn state(&self) -> ProcessState {
        self.state
    }
}

impl Drop for Process {
    // No error handling; "You'll just have to believe in your destructor"
    fn drop(&mut self) {
        if self.attached {
            // Must be stopped before you can ptrace::detach
            if self.state.is_running() {
                let _ = kill(self.pid, Signal::SIGSTOP);
                let _ = waitpid(self.pid, None);
            }
        }

        let _ = ptrace::detach(self.pid, None);
        let _ = kill(self.pid, Signal::SIGCONT);

        if self.terminate_on_end {
            let _ = kill(self.pid, Signal::SIGKILL);
            let _ = waitpid(self.pid, None);
        }
    }
}
