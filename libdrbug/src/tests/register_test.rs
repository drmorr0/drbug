use std::str::from_utf8;

use super::*;
use crate::pipe::Pipe;
use crate::register::info::{
    RegisterId,
    register_info_by_id,
};

#[rstest]
fn test_write_registers() {
    let mut channel = Pipe::new_exec_safe().unwrap();
    let mut proc = Process::launch(
        WRITE_TEST_BINARY,
        ProcessOptions {
            stdout: channel.take_writer().map(|w| w.into()),
            ..Default::default()
        },
    )
    .unwrap();
    proc.resume().unwrap();
    proc.wait_on_signal().unwrap();

    // It would be cool if we could test reading/writing to all the registers
    // but that's going to be an annoying bit of code to write, so instead we'll
    // just follow the book and test a representative from each class.
    //
    // We use these individual blocks to drop the mutable borrows between tests.
    {
        let regs = proc.get_registers_mut();
        let info = register_info_by_id(&RegisterId::rsi);
        regs.write(info, RegisterValue::U64(0xcafecafe)).unwrap();
        proc.resume().unwrap();
        proc.wait_on_signal().unwrap();

        let output = channel.read().unwrap();
        assert_eq!(from_utf8(&output).unwrap(), "0xcafecafe");
    }


    {
        let regs = proc.get_registers_mut();
        let info = register_info_by_id(&RegisterId::mm0);
        regs.write(info, RegisterValue::U64(0xba5eba11)).unwrap();
        proc.resume().unwrap();
        proc.wait_on_signal().unwrap();

        let output = channel.read().unwrap();
        assert_eq!(from_utf8(&output).unwrap(), "0xba5eba11");
    }

    {
        let regs = proc.get_registers_mut();
        let info = register_info_by_id(&RegisterId::xmm0);
        regs.write(info, RegisterValue::F64(42.24)).unwrap();
        proc.resume().unwrap();
        proc.wait_on_signal().unwrap();

        let output = channel.read().unwrap();
        assert_eq!(from_utf8(&output).unwrap(), "42.24");
    }

    // long double currently unsupported
    // {
    //     let regs = proc.get_registers_mut();
    //     regs.write_by_id(RegisterId::st0, RegisterValue::F64(42.24)).unwrap();
    //     regs.write_by_id(RegisterId::fsw, RegisterValue::U16(0b0011100000000000))
    //         .unwrap();
    //     regs.write_by_id(RegisterId::ftw, RegisterValue::U16(0b0011111111111111))
    //         .unwrap();
    //     proc.resume().unwrap();
    //     proc.wait_on_signal().unwrap();

    //     let output = channel.read().unwrap();
    //     assert_eq!(from_utf8(&output).unwrap(), "42.24");
    // }
}

#[rstest]
fn test_read_registers() {
    let mut proc = Process::launch(READ_TEST_BINARY, Default::default()).unwrap();
    {
        proc.resume().unwrap();
        proc.wait_on_signal().unwrap();

        let regs = proc.get_registers();
        let info = register_info_by_id(&RegisterId::r13);
        let val = regs.read(info).unwrap();
        assert_eq!(val, RegisterValue::U64(0xcafecafe));
    }

    {
        proc.resume().unwrap();
        proc.wait_on_signal().unwrap();

        let regs = proc.get_registers();
        let info = register_info_by_id(&RegisterId::r13b);
        let val = regs.read(info).unwrap();
        assert_eq!(val, RegisterValue::U8(42));
    }

    {
        proc.resume().unwrap();
        proc.wait_on_signal().unwrap();

        let regs = proc.get_registers();
        let info = register_info_by_id(&RegisterId::mm0);
        let val = regs.read(info).unwrap();
        assert_eq!(val, RegisterValue::B64([0x11, 0xba, 0x5e, 0xba, 0x11, 0xba, 0x5e, 0xba]));
    }

    {
        proc.resume().unwrap();
        proc.wait_on_signal().unwrap();

        let regs = proc.get_registers();
        let info = register_info_by_id(&RegisterId::xmm0);
        let val = regs.read(info).unwrap();
        assert_eq!(val, RegisterValue::F64(64.125));
    }

    // long double currently unsupported
    // {
    //     proc.resume().unwrap();
    //     proc.wait_on_signal().unwrap();

    //     let regs = proc.get_registers();
    //     let val = regs.read_by_id(RegisterId::st0).unwrap();
    //     assert_eq!(val, RegisterValue::F64(64.125));
    // }
}
