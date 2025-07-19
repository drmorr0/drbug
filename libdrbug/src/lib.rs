mod address;
mod errors;
mod pipe;
mod process;
mod reg;
mod util;

pub use crate::errors::*;

type Empty = DrbugResult<()>;
type Byte64 = [u8; 8];
type Byte128 = [u8; 16];

pub mod prelude {
    pub use crate::address::VirtAddr;
    pub use crate::process::{
        Process,
        ProcessOptions,
    };
    pub use crate::reg::info::{
        RegisterFormat,
        RegisterInfo,
        RegisterType,
        register_info_by_name,
    };
    pub use crate::reg::value::RegisterValue;
}

#[cfg(test)]
mod tests;
