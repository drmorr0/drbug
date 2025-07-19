pub mod info;
pub mod value;

use std::mem::MaybeUninit;

use libc::user;
use nix::sys::ptrace;
use nix::sys::ptrace::AddressType;
use nix::unistd::Pid;

use self::info::{
    DEBUG_REGISTER_IDS,
    REGISTER_INFOS,
    RegisterFormat,
    RegisterInfo,
    RegisterType,
    register_info_by_id,
};
use self::value::RegisterValue;
use crate::util::{
    as_bytes,
    as_bytes_mut,
    copy_bytes,
};
use crate::{
    Byte128,
    DrbugError,
    DrbugResult,
    Empty,
    ptrace_error,
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

    // N.B. The read_* functions are reading the "cached" register values in the Registers.data
    // field; they are _not_ reading from the actual registers via ptrace.  That is done in the
    // `load_all` call below, which is executed whenever the process halts; however, there is a
    // possibility of a state mismatch if we try to read from a register and somehow the "load_all"
    // call hasn't been done yet.
    pub fn read_group(&self, group: Option<RegisterType>) -> DrbugResult<Vec<(&'static str, Option<RegisterValue>)>> {
        REGISTER_INFOS
            .iter()
            .filter_map(|info| {
                if group.is_none_or(|g| info.type_ == g) {
                    if info.format == RegisterFormat::LongDouble {
                        Some(Ok((info.name, None)))
                    } else {
                        Some(self.read(info).map(|v| (info.name, Some(v))))
                    }
                } else {
                    None
                }
            })
            .collect() // collect pulls a Vec of results into a result of vec
    }

    pub fn read(&self, info: &RegisterInfo) -> DrbugResult<RegisterValue> {
        // SAFETY: self.data is #[repr(C)], is not null, and valid for reads; it will not be
        // mutated while in this block, and the total size is less than isize::MAX
        let bytes: &[u8] = as_bytes(&self.data);
        let res = match info.format {
            RegisterFormat::Uint => match info.size {
                1 => RegisterValue::U8(bytes[info.offset]),
                2 => RegisterValue::U16(u16::from_le_bytes(bytes[info.offset..info.offset + info.size].try_into()?)),
                4 => RegisterValue::U32(u32::from_le_bytes(bytes[info.offset..info.offset + info.size].try_into()?)),
                8 => RegisterValue::U64(u64::from_le_bytes(bytes[info.offset..info.offset + info.size].try_into()?)),
                _ => return Err(DrbugError::InvalidRegisterSize(info.size)),
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
                _ => return Err(DrbugError::InvalidRegisterSize(info.size)),
            },
        };
        Ok(res)
    }

    pub fn write(&mut self, info: &RegisterInfo, val: RegisterValue) -> Empty {
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

    pub(crate) fn load_all(&mut self) -> Empty {
        self.data.regs = ptrace_error!(getregs(self.pid))?;
        self.data.i387 = ptrace_error!(getfpregs(self.pid))?;
        for (i, dr) in DEBUG_REGISTER_IDS.iter().enumerate() {
            let info = register_info_by_id(dr);
            let val = ptrace_error!(read_user(self.pid, info.offset as AddressType))?;
            self.data.u_debugreg[i] = val as u64;
        }
        Ok(())
    }

    #[allow(dead_code)] // will use this in a later chapter
    fn commit_gprs(&self) -> Empty {
        ptrace_error!(setregs(self.pid, self.data.regs))
    }

    fn commit_fprs(&self) -> Empty {
        ptrace_error!(setfpregs(self.pid, self.data.i387))
    }

    fn commit_user_area(&mut self, offset: usize, wide_val_bytes: &Byte128) -> Empty {
        let aligned_offset = offset & !0b111;
        ptrace_error!(write_user(
            self.pid,
            aligned_offset as AddressType,
            i64::from_le_bytes(wide_val_bytes[..8].try_into().unwrap()),
        ))
    }
}

fn widen(val: &RegisterValue, info: &RegisterInfo) -> DrbugResult<Byte128> {
    if val.size() > info.size {
        return Err(DrbugError::InvalidRegisterValue(val.clone()));
    }

    if val.is_floating_point() {
        if info.format == RegisterFormat::DoubleFloat {
            return val.cast_to_bytes128::<f64>();
        } else if info.format == RegisterFormat::LongDouble {
            unimplemented!("f80 floating point values not supported");
        }
    } else if val.is_signed() && info.format == RegisterFormat::Uint {
        return match info.size {
            1 => val.cast_to_bytes128::<i8>(),
            2 => val.cast_to_bytes128::<i16>(),
            4 => val.cast_to_bytes128::<i32>(),
            8 => val.cast_to_bytes128::<i64>(),
            _ => Err(DrbugError::InvalidRegisterSize(info.size)),
        };
    }

    Ok(val.into())
}
