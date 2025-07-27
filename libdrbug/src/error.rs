use thiserror::Error;

use crate::address::VirtAddr;
use crate::register::value::RegisterValue;

pub type DrbugResult<T> = Result<T, DrbugError>;

#[derive(Debug, Error)]
pub enum DrbugError {
    #[error("breakpoint site {0} exists at address: {1}")]
    BreakpointSiteExists(usize, VirtAddr),

    #[error("conversion from byte slice failed: {0}")]
    ByteSliceConversionError(#[from] std::array::TryFromSliceError),

    #[error("null byte found in string")]
    CStringNullFound(#[from] std::ffi::NulError),

    #[error("child process failed: {0}")]
    ChildProcessFailed(String),

    #[error("invalid register name: {0}")]
    InvalidRegisterName(String),

    #[error("invalid register size: {0}")]
    InvalidRegisterSize(usize),

    #[error("invalid register value: {0}")]
    InvalidRegisterValue(RegisterValue),

    #[error("i/o error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("long double (f80) type not currently supported")]
    LongDoubleUnsupported,

    #[error("parse error: {0}")]
    ParseError(#[from] std::num::ParseIntError),

    #[error("pipe closed")]
    PipeClosed,

    #[error("{1} failed (errno: {0})")]
    SyscallFailed(&'static str, nix::Error),

    #[error("conversion from {0} to {1} failed")]
    RegisterValueConversionFailed(&'static str, RegisterValue),
}

#[macro_export]
macro_rules! syscall_error {
    ($syscall:ident($($args:expr),* $(,)?)) => { $syscall($($args),*).map_err(|e| DrbugError::SyscallFailed(stringify!($syscall), e)) };
    ($prefix:ident::$syscall:ident($($args:expr),* $(,)?)) => { $prefix::$syscall($($args),*).map_err(|e| DrbugError::SyscallFailed(stringify!($syscall), e)) };
}
