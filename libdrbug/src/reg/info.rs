use std::collections::HashMap;
use std::mem::offset_of;
use std::sync::LazyLock;

use libc::{
    user,
    user_fpregs_struct,
    user_regs_struct,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegisterType {
    General,
    SubGeneral,
    FloatingPoint,
    Debug,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegisterFormat {
    Uint,
    #[allow(dead_code)] // Not used in the debugger book
    DoubleFloat,
    LongDouble,
    Vector,
}

pub struct RegisterInfo {
    pub id: RegisterId, // defined by the giant macro below
    #[allow(dead_code)]
    pub name: &'static str,
    #[allow(dead_code)]
    pub dwarf_id: Option<u32>,
    #[allow(dead_code)]
    pub size: usize,
    pub offset: usize,
    #[allow(dead_code)]
    pub type_: RegisterType,
    #[allow(dead_code)]
    pub format: RegisterFormat,
}

macro_rules! gpr_offset {
    ($reg:expr) => {
        offset_of!(user, regs) + offset_of!(user_regs_struct, $reg)
    };
}

macro_rules! fpr_offset {
    ($reg:expr) => {
        offset_of!(user, i387) + offset_of!(user_fpregs_struct, $reg)
    };
}

macro_rules! debug_offset {
    ($id:literal) => {
        offset_of!(user, u_debugreg) + $id * 8
    };
}

pub const fn size_of_return_value<F, T, U>(_f: &F) -> usize
where
    F: FnOnce(T) -> U,
{
    size_of::<U>()
}

// This technique taken from here:
// https://internals.rust-lang.org/t/official-way-to-get-the-size-of-a-field/22123
macro_rules! fpr_size {
    ($reg:ident) => {
        size_of_return_value(&|s: user_fpregs_struct| s.$reg)
    };
}

macro_rules! define_registers {
    ($(($name:ident, $($args:tt),+): $reg_type:ident),* $(,)?) => {
        #[allow(non_camel_case_types)]
        #[allow(dead_code)]
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum RegisterId {
            $($name),*
        }

        #[allow(dead_code)]
        pub const REGISTER_INFOS: &[RegisterInfo] = &[
            $(define_registers!(@generate_call $reg_type, $name, $($args),+),)*
        ];
    };

    // Helper rules to generate the appropriate macro call based on register type
    (@generate_call GP64, $name:ident, None) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: None,
            size: 8,
            offset: gpr_offset!($name),
            type_: RegisterType::General,
            format: RegisterFormat::Uint,
        }
    };

    (@generate_call GP64, $name:ident, $dwarf_id:literal) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: Some($dwarf_id),
            size: 8,
            offset: gpr_offset!($name),
            type_: RegisterType::General,
            format: RegisterFormat::Uint,
        }
    };

    (@generate_call GP32, $name:ident, $super:ident) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: None,
            size: 4,
            offset: gpr_offset!($super),
            type_: RegisterType::SubGeneral,
            format: RegisterFormat::Uint,
        }
    };

    (@generate_call GP16, $name:ident, $super:ident) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: None,
            size: 2,
            offset: gpr_offset!($super),
            type_: RegisterType::SubGeneral,
            format: RegisterFormat::Uint,
        }
    };

    (@generate_call GP8H, $name:ident, $super:ident) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: None,
            size: 1,
            offset: gpr_offset!($super),
            type_: RegisterType::SubGeneral,
            format: RegisterFormat::Uint,
        }
    };

    (@generate_call GP8L, $name:ident, $super:ident) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: None,
            size: 1,
            offset: gpr_offset!($super),
            type_: RegisterType::SubGeneral,
            format: RegisterFormat::Uint,
        }
    };

    (@generate_call FPR, $name:ident, None, $user_name:ident) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: None,
            size: fpr_size!($user_name),
            offset: fpr_offset!($user_name),
            type_: RegisterType::FloatingPoint,
            format: RegisterFormat::Uint,
        }
    };

    (@generate_call FPR, $name:ident, $dwarf_id:expr, $user_name:ident) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: Some($dwarf_id),
            size: fpr_size!($user_name),
            offset: fpr_offset!($user_name),
            type_: RegisterType::FloatingPoint,
            format: RegisterFormat::Uint,
        }
    };

    (@generate_call FP_ST, $name:ident, $id:tt) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: Some(33 + $id),
            size: 16,
            offset: fpr_offset!(st_space) + $id * 16,
            type_: RegisterType::FloatingPoint,
            format: RegisterFormat::LongDouble,
        }
    };

	// N.B. The mm registers map to the same memory as the st registers
    (@generate_call FP_MM, $name:ident, $id:tt) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: Some(41 + $id),
            size: 8,
            offset: fpr_offset!(st_space) + $id * 16,
            type_: RegisterType::FloatingPoint,
            format: RegisterFormat::Vector,
        }
    };

    (@generate_call FP_XMM, $name:ident, $id:tt) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: Some(17 + $id),
            size: 16,
            offset: fpr_offset!(xmm_space) + $id * 16,
            type_: RegisterType::FloatingPoint,
            format: RegisterFormat::Vector,
        }
    };

    (@generate_call DEBUG, $name:ident, $id:tt) => {
        RegisterInfo {
            id: RegisterId::$name,
            name: stringify!($name),
            dwarf_id: None,
            size: 8,
            offset: debug_offset!($id),
            type_: RegisterType::Debug,
            format: RegisterFormat::Uint,
        }
    };
}

define_registers!(
    (rax, 0): GP64,
    (rdx, 1): GP64,
    (rcx, 2): GP64,
    (rbx, 3): GP64,
    (rsi, 4): GP64,
    (rdi, 5): GP64,
    (rbp, 6): GP64,
    (rsp, 7): GP64,
    (r8, 8): GP64,
    (r9, 9): GP64,
    (r10, 10): GP64,
    (r11, 11): GP64,
    (r12, 12): GP64,
    (r13, 13): GP64,
    (r14, 14): GP64,
    (r15, 15): GP64,
    (rip, 16): GP64,
    (eflags, 49): GP64,
    (cs, 51): GP64,
    (fs, 54): GP64,
    (gs, 55): GP64,
    (ss, 52): GP64,
    (ds, 53): GP64,
    (es, 50): GP64,
    (orig_rax, None): GP64,

    (eax, rax): GP32,
    (edx, rdx): GP32,
    (ecx, rcx): GP32,
    (ebx, rbx): GP32,
    (esi, rsi): GP32,
    (edi, rdi): GP32,
    (ebp, rbp): GP32,
    (esp, rsp): GP32,
    (r8d, r8): GP32,
    (r9d, r9): GP32,
    (r10d, r10): GP32,
    (r11d, r11): GP32,
    (r12d, r12): GP32,
    (r13d, r13): GP32,
    (r14d, r14): GP32,
    (r15d, r15): GP32,

    (ax, rax): GP16,
    (dx, rdx): GP16,
    (cx, rcx): GP16,
    (bx, rbx): GP16,
    (si, rsi): GP16,
    (di, rdi): GP16,
    (bp, rbp): GP16,
    (sp, rsp): GP16,
    (r8w, r8): GP16,
    (r9w, r9): GP16,
    (r10w, r10): GP16,
    (r11w, r11): GP16,
    (r12w, r12): GP16,
    (r13w, r13): GP16,
    (r14w, r14): GP16,
    (r15w, r15): GP16,

    (ah, rax): GP8H,
    (dh, rdx): GP8H,
    (ch, rcx): GP8H,
    (bh, rbx): GP8H,

    (al, rax): GP8L,
    (dl, rdx): GP8L,
    (cl, rcx): GP8L,
    (bl, rbx): GP8L,
    (sil, rsi): GP8L,
    (dil, rdi): GP8L,
    (bpl, rbp): GP8L,
    (spl, rsp): GP8L,
    (r8b, r8): GP8L,
    (r9b, r9): GP8L,
    (r10b, r10): GP8L,
    (r11b, r11): GP8L,
    (r12b, r12): GP8L,
    (r13b, r13): GP8L,
    (r14b, r14): GP8L,
    (r15b, r15): GP8L,

    (fcw, 65, cwd): FPR,
    (fsw, 66, swd): FPR,
    (ftw, None, ftw): FPR,
    (fop, None, fop): FPR,
    (frip, None, rip): FPR,
    (frdp, None, rdp): FPR,
    (mxcsr, 64, mxcsr): FPR,
    (mxcsrmask, None, mxcr_mask): FPR,

    (st0, 0): FP_ST,
    (st1, 1): FP_ST,
    (st2, 2): FP_ST,
    (st3, 3): FP_ST,
    (st4, 4): FP_ST,
    (st5, 5): FP_ST,
    (st6, 6): FP_ST,
    (st7, 7): FP_ST,

    (mm0, 0): FP_MM,
    (mm1, 1): FP_MM,
    (mm2, 2): FP_MM,
    (mm3, 3): FP_MM,
    (mm4, 4): FP_MM,
    (mm5, 5): FP_MM,
    (mm6, 6): FP_MM,
    (mm7, 7): FP_MM,

    (xmm0, 0): FP_XMM,
    (xmm1, 1): FP_XMM,
    (xmm2, 2): FP_XMM,
    (xmm3, 3): FP_XMM,
    (xmm4, 4): FP_XMM,
    (xmm5, 5): FP_XMM,
    (xmm6, 6): FP_XMM,
    (xmm7, 7): FP_XMM,
    (xmm8, 8): FP_XMM,
    (xmm9, 9): FP_XMM,
    (xmm10, 10): FP_XMM,
    (xmm11, 11): FP_XMM,
    (xmm12, 12): FP_XMM,
    (xmm13, 13): FP_XMM,
    (xmm14, 14): FP_XMM,
    (xmm15, 15): FP_XMM,

    (dr0, 0): DEBUG,
    (dr1, 1): DEBUG,
    (dr2, 2): DEBUG,
    (dr3, 3): DEBUG,
    (dr4, 4): DEBUG,
    (dr5, 5): DEBUG,
    (dr6, 6): DEBUG,
    (dr7, 7): DEBUG,
);

pub const DEBUG_REGISTER_IDS: [RegisterId; 8] = [
    RegisterId::dr0,
    RegisterId::dr1,
    RegisterId::dr2,
    RegisterId::dr3,
    RegisterId::dr4,
    RegisterId::dr5,
    RegisterId::dr6,
    RegisterId::dr7,
];

pub(super) static REG_INFOS_BY_ID: LazyLock<HashMap<RegisterId, &RegisterInfo>> =
    LazyLock::new(|| REGISTER_INFOS.iter().map(|rinfo| (rinfo.id, rinfo)).collect::<HashMap<_, _>>());

#[allow(dead_code)]
pub(super) static REG_INFOS_BY_NAME: LazyLock<HashMap<&'static str, &RegisterInfo>> = LazyLock::new(|| {
    REGISTER_INFOS
        .iter()
        .map(|rinfo| (rinfo.name, rinfo))
        .collect::<HashMap<_, _>>()
});

#[allow(dead_code)]
pub(super) static REG_INFOS_BY_DWARF_ID: LazyLock<HashMap<u32, &RegisterInfo>> = LazyLock::new(|| {
    REGISTER_INFOS
        .iter()
        .filter_map(|rinfo| rinfo.dwarf_id.map(|did| (did, rinfo)))
        .collect::<HashMap<_, _>>()
});

pub fn register_info_by_id(id: &RegisterId) -> &'static RegisterInfo {
    REG_INFOS_BY_ID.get(id).expect("invalid register id")
}
