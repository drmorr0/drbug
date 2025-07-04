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
    pub use crate::process::Process;
}

#[cfg(test)]
mod tests;
