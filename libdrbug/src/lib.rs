#[allow(clippy::missing_safety_doc)]
mod address;
mod breakpoint;
mod error;
mod pipe;
mod process;
mod register;
mod util;

pub use crate::error::*;

type Empty = DrbugResult<()>;
type Byte64 = [u8; 8];
type Byte128 = [u8; 16];

pub mod prelude {
    pub use crate::address::VirtAddr;
    pub use crate::breakpoint::Breakable;
    pub use crate::process::{
        Process,
        ProcessOptions,
    };
    pub use crate::register::info::{
        RegisterFormat,
        RegisterInfo,
        RegisterType,
        register_info_by_name,
    };
    pub use crate::register::value::RegisterValue;
}

#[cfg(test)]
mod tests;
