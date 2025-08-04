use std::collections::{
    HashMap,
    hash_map,
};

use super::Breakable;
use crate::Empty;
use crate::address::VirtAddr;

#[derive(Debug)]
pub struct BreakList<T: Breakable> {
    breaks: HashMap<usize, T>,
    rindex: HashMap<VirtAddr, usize>,
}

impl<T: Breakable + Clone> BreakList<T> {
    pub(crate) fn new() -> Self {
        BreakList { breaks: HashMap::new(), rindex: HashMap::new() }
    }

    pub fn add(&mut self, b: T) {
        self.rindex.insert(b.addr(), b.id());
        self.breaks.insert(b.id(), b);
    }

    pub fn breakable_enabled_at(&self, addr: &VirtAddr) -> bool {
        self.get_by_addr(addr).is_some_and(|b| b.enabled())
    }

    pub fn get(&self, id: &usize) -> Option<T> {
        self.breaks.get(id).cloned()
    }

    pub fn get_by_addr(&self, addr: &VirtAddr) -> Option<T> {
        self.rindex.get(addr).and_then(|ind| self.breaks.get(ind).cloned())
    }

    pub fn is_empty(&self) -> bool {
        self.breaks.is_empty()
    }

    pub fn iter(&self) -> hash_map::Iter<usize, T> {
        self.breaks.iter()
    }

    pub fn len(&self) -> usize {
        self.breaks.len()
    }

    pub fn remove(&mut self, id: &usize) -> Empty {
        if let Some(mut bp) = self.breaks.remove(id) {
            bp.disable()?;
            self.rindex.remove(&bp.addr());
        }
        Ok(())
    }

    pub fn remove_by_addr(&mut self, addr: &VirtAddr) -> Empty {
        if let Some(ind) = self.rindex.remove(addr) { self.remove(&ind) } else { Ok(()) }
    }
}
