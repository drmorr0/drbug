use std::fmt;

use anyhow::bail;

use crate::prelude::*;
use crate::util::copy_bytes;

#[derive(Debug)]
pub enum RegisterValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    B64(Byte64),
    B128(Byte128),
}

impl RegisterValue {
    pub fn size(&self) -> usize {
        match &self {
            RegisterValue::U8(_) => size_of::<u8>(),
            RegisterValue::U16(_) => size_of::<u16>(),
            RegisterValue::U32(_) => size_of::<u32>(),
            RegisterValue::U64(_) => size_of::<u64>(),
            RegisterValue::I8(_) => size_of::<i8>(),
            RegisterValue::I16(_) => size_of::<i16>(),
            RegisterValue::I32(_) => size_of::<i32>(),
            RegisterValue::I64(_) => size_of::<i64>(),
            RegisterValue::F32(_) => size_of::<f32>(),
            RegisterValue::F64(_) => size_of::<f64>(),
            RegisterValue::B64(_) => 8,
            RegisterValue::B128(_) => 16,
        }
    }

    pub fn is_floating_point(&self) -> bool {
        matches!(self, RegisterValue::F32(_) | RegisterValue::F64(_))
    }

    pub fn is_signed(&self) -> bool {
        matches!(self, RegisterValue::I8(_) | RegisterValue::I16(_) | RegisterValue::I32(_) | RegisterValue::I64(_))
    }

    pub fn cast_to_bytes128<T: RegisterValueTarget>(&self) -> anyhow::Result<Byte128> {
        Ok(T::from_register_value(self)?.into())
    }
}

impl fmt::Display for RegisterValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegisterValue::U8(val) => write!(f, "0x{val:02x}"),
            RegisterValue::U16(val) => write!(f, "0x{val:04x}"),
            RegisterValue::U32(val) => write!(f, "0x{val:08x}"),
            RegisterValue::U64(val) => write!(f, "0x{val:016x}"),
            RegisterValue::I8(val) => write!(f, "0x{val:02x}"),
            RegisterValue::I16(val) => write!(f, "0x{val:04x}"),
            RegisterValue::I32(val) => write!(f, "0x{val:08x}"),
            RegisterValue::I64(val) => write!(f, "0x{val:016x}"),
            RegisterValue::F32(val) => write!(f, "{val}"),
            RegisterValue::F64(val) => write!(f, "{val}"),
            RegisterValue::B64(val) => {
                let bytes: Vec<String> = val.iter().map(|b| format!("0x{b:02x}")).collect();
                write!(f, "[{}]", bytes.join(", "))
            },
            RegisterValue::B128(val) => {
                let bytes: Vec<String> = val.iter().map(|b| format!("0x{b:02x}")).collect();
                write!(f, "[{}]", bytes.join(", "))
            },
        }
    }
}


pub trait RegisterValueTarget {
    fn from_register_value(value: &RegisterValue) -> anyhow::Result<RegisterValue>;
}

impl RegisterValueTarget for i16 {
    fn from_register_value(value: &RegisterValue) -> anyhow::Result<RegisterValue> {
        match value {
            RegisterValue::I8(v) => Ok(RegisterValue::I16(*v as i16)),
            RegisterValue::I16(v) => Ok(RegisterValue::I16(*v)),
            x => bail!("invalid conversion to i16 from {x:?}"),
        }
    }
}

impl RegisterValueTarget for i32 {
    fn from_register_value(value: &RegisterValue) -> anyhow::Result<RegisterValue> {
        match value {
            RegisterValue::I8(v) => Ok(RegisterValue::I32(*v as i32)),
            RegisterValue::I16(v) => Ok(RegisterValue::I32(*v as i32)),
            RegisterValue::I32(v) => Ok(RegisterValue::I32(*v)),
            x => bail!("invalid conversion to i32 from {x:?}"),
        }
    }
}

impl RegisterValueTarget for i64 {
    fn from_register_value(value: &RegisterValue) -> anyhow::Result<RegisterValue> {
        match value {
            RegisterValue::I8(v) => Ok(RegisterValue::I64(*v as i64)),
            RegisterValue::I16(v) => Ok(RegisterValue::I64(*v as i64)),
            RegisterValue::I32(v) => Ok(RegisterValue::I64(*v as i64)),
            RegisterValue::I64(v) => Ok(RegisterValue::I64(*v)),
            x => bail!("invalid conversion to i64 from {x:?}"),
        }
    }
}

impl RegisterValueTarget for f64 {
    fn from_register_value(value: &RegisterValue) -> anyhow::Result<RegisterValue> {
        match value {
            RegisterValue::F32(v) => Ok(RegisterValue::F64(*v as f64)),
            RegisterValue::F64(v) => Ok(RegisterValue::F64(*v)),
            x => bail!("invalid conversion to f64 from {x:?}"),
        }
    }
}

impl From<&RegisterValue> for Byte128 {
    fn from(val: &RegisterValue) -> Self {
        let mut ret = [0; 16];
        match val {
            RegisterValue::U8(v) => copy_bytes(&mut ret, v),
            RegisterValue::U16(v) => copy_bytes(&mut ret, v),
            RegisterValue::U32(v) => copy_bytes(&mut ret, v),
            RegisterValue::U64(v) => copy_bytes(&mut ret, v),
            RegisterValue::I8(v) => copy_bytes(&mut ret, v),
            RegisterValue::I16(v) => copy_bytes(&mut ret, v),
            RegisterValue::I32(v) => copy_bytes(&mut ret, v),
            RegisterValue::I64(v) => copy_bytes(&mut ret, v),
            RegisterValue::F32(v) => copy_bytes(&mut ret, v),
            RegisterValue::F64(v) => copy_bytes(&mut ret, v),
            RegisterValue::B64(v) => copy_bytes(&mut ret, v),
            RegisterValue::B128(v) => copy_bytes(&mut ret, v),
        }
        ret
    }
}

impl From<RegisterValue> for Byte128 {
    fn from(val: RegisterValue) -> Self {
        Byte128::from(&val)
    }
}

impl PartialEq for RegisterValue {
    fn eq(&self, other: &Self) -> bool {
        Into::<Byte128>::into(self) == Into::<Byte128>::into(other)
    }
}

impl Eq for RegisterValue {}
