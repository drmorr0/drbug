use std::cmp::min;
use std::io::IoSliceMut;

use nix::sys::ptrace;
use nix::sys::uio::{
    RemoteIoVec,
    process_vm_readv,
};

use super::Process;
use crate::address::VirtAddr;
use crate::{
    DrbugError,
    DrbugResult,
    Empty,
    syscall_error,
};

impl Process {
    pub fn read_memory(&self, mut addr: VirtAddr, mut size: usize) -> DrbugResult<Vec<u8>> {
        let mut buf = vec![0u8; size];
        let local_iov = IoSliceMut::new(&mut buf);
        let mut remote_iovs = vec![];

        while size > 0 {
            // Read on memory page boundaries to make sure we don't move into memory we
            // don't have permissions for
            //
            // Default page size on Linux is 4KiB, aka 0x1000 bytes; I suppose this will
            // break if we run it on a system with a non-standard page size (I think that's
            // configurable?  Honestly who knows)
            let up_to_next_page = 0x1000 - (addr.0 & 0xfff) as usize;
            let chunk_size = min(size, up_to_next_page);
            remote_iovs.push(RemoteIoVec { base: addr.0 as usize, len: chunk_size });
            size -= chunk_size;
            addr = addr.add(chunk_size);
        }

        syscall_error!(process_vm_readv(self.pid, &mut [local_iov], &remote_iovs))?;
        Ok(buf)
    }

    pub fn write_memory(&mut self, addr: VirtAddr, data: &[u8]) -> Empty {
        let mut written = 0usize;

        // Have to write one byte at a time because /proc/pid/mem sucks on WSL :sigh:
        while let remaining = data.len().saturating_sub(written)
            && remaining > 0
        {
            let curr_addr = addr.add(written);
            let mut word_bytes = [0u8; 8];
            if remaining >= 8 {
                word_bytes.copy_from_slice(&data[written..written + 8]);
            } else {
                let old_bytes = self.read_memory(curr_addr, 8)?;
                word_bytes[0..remaining].copy_from_slice(&data[written..written + remaining]);
                word_bytes[remaining..].copy_from_slice(&old_bytes[remaining..]);
            }

            let word = i64::from_le_bytes(word_bytes);
            syscall_error!(ptrace::write(self.pid, unsafe { curr_addr.into_void_ptr() }, word))?;
            written += 8;
        }
        Ok(())
    }
}
