use super::Process;
use crate::address::VirtAddr;
use crate::breakpoint::{
    BreakList,
    Breakable,
    BreakpointSite,
};
use crate::{
    DrbugError,
    DrbugResult,
};

impl Process {
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

        let site = BreakpointSite::new(self.pid, addr);
        self.breakpoint_sites.add(site.clone());
        Ok(site)
    }
}
