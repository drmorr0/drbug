use std::str::FromStr;

use derive_more::{
    Add,
    Display,
    LowerHex,
    Sub,
};
use libc::c_void;

use crate::{
    DrbugError,
    DrbugResult,
};

#[derive(Add, Debug, Display, Clone, Copy, Eq, Hash, LowerHex, PartialEq, Sub)]
#[display("0x{_0:016x}")]
pub struct VirtAddr(pub u64);

impl VirtAddr {
    // SAFETY: we can't really just be casting anything to random addresses and
    // expect it to work
    pub unsafe fn into_void_ptr(&self) -> *mut c_void {
        self.0 as *mut c_void
    }

    pub fn add(&self, size: usize) -> Self {
        Self(self.0 + size as u64)
    }

    pub fn decrement(&mut self) {
        self.0 -= 1;
    }
}

impl FromStr for VirtAddr {
    type Err = DrbugError;

    fn from_str(s: &str) -> DrbugResult<Self> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        u64::from_str_radix(s, 16).map(VirtAddr).map_err(|e| e.into())
    }
}
