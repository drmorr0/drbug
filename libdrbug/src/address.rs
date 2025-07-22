use derive_more::{
    Add,
    Display,
};

#[derive(Add, Debug, Display, Clone, Copy, Eq, Hash, PartialEq)]
#[display("0x{_0:016x}")]
pub struct VirtAddr(pub u64);
