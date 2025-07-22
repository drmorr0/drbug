use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};

use super::Breakable;
use crate::address::VirtAddr;

static BP_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreakpointSite {
    id: usize,
    addr: VirtAddr,
    is_enabled: bool,
    saved_data: u8,
}

impl BreakpointSite {
    pub(crate) fn new(addr: VirtAddr) -> Self {
        let next_id = BP_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
        BreakpointSite {
            id: next_id,
            addr,
            is_enabled: false,
            saved_data: 0,
        }
    }
}

impl Breakable for BreakpointSite {
    fn addr(&self) -> VirtAddr {
        self.addr
    }

    fn enabled(&self) -> bool {
        self.is_enabled
    }

    fn id(&self) -> usize {
        self.id
    }
}
