mod list;
mod site;

pub use self::list::BreakList;
pub use self::site::BreakpointSite;
use crate::Empty;
use crate::address::VirtAddr;

pub trait Breakable {
    fn addr(&self) -> VirtAddr;
    fn disable(&mut self) -> Empty;
    fn enable(&mut self) -> Empty;
    fn enabled(&self) -> bool;
    fn id(&self) -> usize;
}
