mod list;
mod site;

pub use self::list::BreakList;
pub use self::site::BreakpointSite;
use crate::address::VirtAddr;

pub trait Breakable {
    fn addr(&self) -> VirtAddr;
    fn enabled(&self) -> bool;
    fn id(&self) -> usize;
}
