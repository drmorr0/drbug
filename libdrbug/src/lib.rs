mod pipe;
mod process;
mod reg;
mod util;

pub type Empty = anyhow::Result<()>;
pub type Byte64 = [u8; 8];
pub type Byte128 = [u8; 16];

pub mod prelude {
    pub use super::*;
    pub use crate::pipe::Pipe;
    pub use crate::process::{
        Process,
        ProcessOptions,
    };
    pub use crate::reg::info::{
        RegisterFormat,
        RegisterId,
        RegisterType,
    };
    pub use crate::reg::value::RegisterValue;
}

#[cfg(test)]
mod tests;
