use std::str::from_utf8;

use super::*;

#[rstest]
fn test_write_gpr() {
    let mut channel = Pipe::new_exec_safe().unwrap();
    let mut proc = Process::launch(
        "../target/asm/reg_write",
        ProcessOptions {
            stdout: channel.take_writer().map(|w| w.into()),
            ..Default::default()
        },
    )
    .unwrap();
    proc.resume().unwrap();
    proc.wait_on_signal().unwrap();

    let regs = proc.get_registers_mut();
    regs.write_by_id(RegisterId::rsi, RegisterValue::U64(0xcafecafe)).unwrap();

    proc.resume().unwrap();
    proc.wait_on_signal().unwrap();

    let output = channel.read().unwrap();
    assert_eq!(from_utf8(&output).unwrap(), "0xcafecafe");
}
