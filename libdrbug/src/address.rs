use std::cmp::{
    Ord,
    Ordering,
    PartialOrd,
};
use std::str::FromStr;

use derive_more::{
    Display,
    Into,
    LowerHex,
};
use libc::c_void;

use crate::{
    DrbugError,
    DrbugResult,
};

#[derive(Debug, Display, Clone, Copy, Eq, Hash, Into, LowerHex, PartialEq)]
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

    pub fn delta(&self, other: VirtAddr) -> Option<usize> {
        self.0.checked_sub(other.0).map(|v| v as usize)
    }
}

impl FromStr for VirtAddr {
    type Err = DrbugError;

    fn from_str(s: &str) -> DrbugResult<Self> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        u64::from_str_radix(s, 16).map(VirtAddr).map_err(|e| e.into())
    }
}

impl Ord for VirtAddr {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for VirtAddr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
