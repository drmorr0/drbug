pub mod info;
pub mod value;

use std::mem::MaybeUninit;

use anyhow::bail;
use libc::user;
use nix::sys::ptrace;
use nix::sys::ptrace::AddressType;
use nix::unistd::Pid;

use self::info::{
    DEBUG_REGISTER_IDS,
    RegisterFormat,
    RegisterInfo,
    RegisterType,
    register_info_by_id,
};
use self::value::RegisterValue;
use crate::prelude::*;
use crate::util::{
    as_bytes,
    as_bytes_mut,
    copy_bytes,
};

#[derive(Debug)]
pub struct Registers {
    pid: Pid,
    data: user,
}

impl Registers {
    pub fn new(pid: Pid) -> Self {
        // SAFETY: I have no idea if this is actually safe (I _think_ it is?) but it's the
        // recommended/only way to construct most of the structs in libc.  Also AFAICT the data
        // struct in the equivalent C code is also uninitialized so maybe it's fine?
        let data = MaybeUninit::zeroed();
        Registers { data: unsafe { data.assume_init() }, pid }
    }

    pub fn read_all(&mut self) -> Empty {
        self.data.regs = ptrace::getregs(self.pid)?;
        self.data.i387 = ptrace::getfpregs(self.pid)?;
        for (i, dr) in DEBUG_REGISTER_IDS.iter().enumerate() {
            let info = register_info_by_id(dr);
            let val = ptrace::read_user(self.pid, info.offset as AddressType)?;
            self.data.u_debugreg[i] = val as u64;
        }
        Ok(())
    }

    pub fn write_by_id(&mut self, id: RegisterId, val: RegisterValue) -> Empty {
        let info = register_info_by_id(&id);
        self.write_single(info, val)
    }

    #[allow(dead_code)]
    fn read_single(&self, info: &RegisterInfo) -> anyhow::Result<RegisterValue> {
        // SAFETY: self.data is #[repr(C)], is not null, and valid for reads; it will not be
        // mutated while in this block, and the total size is less than isize::MAX
        let bytes: &[u8] = as_bytes(&self.data);
        let res = match info.format {
            RegisterFormat::Uint => match info.size {
                1 => RegisterValue::U8(bytes[info.offset]),
                2 => RegisterValue::U16(u16::from_le_bytes(bytes[info.offset..info.offset + info.size].try_into()?)),
                4 => RegisterValue::U32(u32::from_le_bytes(bytes[info.offset..info.offset + info.size].try_into()?)),
                8 => RegisterValue::U64(u64::from_le_bytes(bytes[info.offset..info.offset + info.size].try_into()?)),
                _ => bail!("unexpected register size for Uint format for {}: {}", info.name, info.size),
            },
            RegisterFormat::DoubleFloat => {
                RegisterValue::F64(f64::from_le_bytes(bytes[info.offset..info.offset + info.size].try_into()?))
            },
            RegisterFormat::LongDouble => {
                unimplemented!("f80 floating point values not supported")
            },
            RegisterFormat::Vector => match info.size {
                8 => RegisterValue::B64(bytes[info.offset..info.offset + info.size].try_into()?),
                16 => RegisterValue::B128(bytes[info.offset..info.offset + info.size].try_into()?),
                _ => bail!("unexpected register size for Vector format from {}", info.name),
            },
        };
        Ok(res)
    }

    fn write_single(&mut self, info: &RegisterInfo, val: RegisterValue) -> Empty {
        // SAFETY: self.data is #[repr(C)], is not null, and valid for reads; it will not be
        // read or mutated while in this block, and the total size is less than isize::MAX
        let bytes: &mut [u8] = as_bytes_mut(&mut self.data);

        let wide_val_bytes = widen(&val, info)?;
        copy_bytes(&mut bytes[info.offset..], &wide_val_bytes);

        if info.type_ == RegisterType::FloatingPoint {
            self.commit_fprs()
        } else {
            self.commit_user_area(info.offset, &wide_val_bytes)
        }
    }

    #[allow(dead_code)]
    fn commit_gprs(&self) -> Empty {
        ptrace::setregs(self.pid, self.data.regs).map_err(|e| e.into())
    }

    fn commit_fprs(&self) -> Empty {
        ptrace::setfpregs(self.pid, self.data.i387).map_err(|e| e.into())
    }

    fn commit_user_area(&mut self, offset: usize, wide_val_bytes: &Byte128) -> Empty {
        let aligned_offset = offset & !0b111;
        ptrace::write_user(
            self.pid,
            aligned_offset as AddressType,
            i64::from_le_bytes(wide_val_bytes[..8].try_into().unwrap()),
        )
        .map_err(|e| e.into())
    }
}

fn widen(val: &RegisterValue, info: &RegisterInfo) -> anyhow::Result<Byte128> {
    if val.size() > info.size {
        bail!("value is too big for register {} size: {} > {}", info.name, val.size(), info.size);
    }

    if val.is_floating_point() {
        if info.format == RegisterFormat::DoubleFloat {
            return val.cast_to_bytes128::<f64>();
        } else if info.format == RegisterFormat::LongDouble {
            unimplemented!("f80 floating point values not supported");
        }
    } else if val.is_signed() && info.format == RegisterFormat::Uint {
        match info.size {
            // don't need to specially cast i8 values
            2 => return val.cast_to_bytes128::<i16>(),
            4 => return val.cast_to_bytes128::<i32>(),
            8 => return val.cast_to_bytes128::<i64>(),
            _ => bail!("unexpected register size for Uint format for {}: {}", info.name, info.size),
        }
    }

    Ok(val.into())
}
