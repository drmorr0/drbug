use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};

use nix::sys::ptrace;
use nix::unistd::Pid;

use super::Breakable;
use crate::address::VirtAddr;
use crate::{
    DrbugError,
    Empty,
    syscall_error,
};

static BP_COUNT: AtomicUsize = AtomicUsize::new(0);
const INT3: u64 = 0xcc; // 0xcc is the opcode for the int3 instruction, which is a special interrupt

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreakpointSite {
    id: usize,
    pid: Pid,
    addr: VirtAddr,
    is_enabled: Rc<Cell<bool>>,
    saved_data: Rc<Cell<u8>>,
}

impl BreakpointSite {
    pub(crate) fn new(pid: Pid, addr: VirtAddr) -> Self {
        let next_id = BP_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
        BreakpointSite {
            id: next_id,
            pid,
            addr,
            is_enabled: Rc::new(Cell::new(false)),
            saved_data: Rc::new(Cell::new(0)),
        }
    }
}

impl Breakable for BreakpointSite {
    fn addr(&self) -> VirtAddr {
        self.addr
    }

    fn disable(&mut self) -> Empty {
        if !self.is_enabled.get() {
            return Ok(());
        }

        // SAFETY: presumably we're only setting breakpoints on program locations
        let ptr = unsafe { self.addr.into_void_ptr() };

        let data_with_int3 = syscall_error!(ptrace::read(self.pid, ptr))? as u64;
        let original_data = ((data_with_int3 & !0xff) | self.saved_data.get() as u64) as i64;

        syscall_error!(ptrace::write(self.pid, ptr, original_data))?;

        self.is_enabled.set(false);
        Ok(())
    }

    fn enable(&mut self) -> Empty {
        if self.is_enabled.get() {
            return Ok(());
        }

        // SAFETY: presumably we're only setting breakpoints on program locations
        let ptr = unsafe { self.addr.into_void_ptr() };

        let data = syscall_error!(ptrace::read(self.pid, ptr))? as u64;
        self.saved_data.set((data & 0xff) as u8);
        let data_with_int3 = ((data & !0xff) | INT3) as i64;

        syscall_error!(ptrace::write(self.pid, ptr, data_with_int3))?;

        self.is_enabled.set(true);
        Ok(())
    }

    fn enabled(&self) -> bool {
        self.is_enabled.get()
    }

    fn id(&self) -> usize {
        self.id
    }
}
