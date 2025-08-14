use super::*;
use crate::pipe::Pipe;
use crate::process::Process;
use crate::tests::util::addr_from_bytes;

const TEST_STR: &str = "Hello, drb!";

#[rstest]
fn test_rw_memory() -> Empty {
    let mut channel = Pipe::new().unwrap();
    let opts = ProcessOptions {
        // We don't have to explicitly close the writer, because
        // taking it here drops it from the channel
        stdout: channel.take_writer().map(|w| w.into()),
        ..Default::default()
    };
    let mut proc = Process::launch(MEMORY_PATH, opts).unwrap();

    proc.resume()?;
    proc.wait_on_signal()?;
    let output = channel.read()?;
    let a_addr = addr_from_bytes(&output)?;

    let data = u64::from_le_bytes(proc.read_memory(a_addr, 8)?.try_into().unwrap());
    assert_eq!(data, 0xcafecafe);

    proc.resume()?;
    proc.wait_on_signal()?;

    let output = channel.read()?;
    let b_addr = addr_from_bytes(&output)?;
    proc.write_memory(b_addr, TEST_STR.as_bytes())?;

    proc.resume()?;
    proc.wait_on_signal()?;

    let output = channel.read()?;
    let res = str::from_utf8(&output).unwrap();
    assert_eq!(res, TEST_STR);
    Ok(())
}
