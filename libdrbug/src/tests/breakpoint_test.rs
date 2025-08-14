use std::path::Path;
use std::str;

use nix::sys::signal::Signal;

use super::*;
use crate::DrbugError;
use crate::breakpoint::Breakable;
use crate::pipe::Pipe;
use crate::tests::util::{
    get_entry_point_offset,
    get_load_addr,
};

#[rstest]
fn test_create_breakpoint_site() -> Empty {
    let mut proc = Process::launch(LOOP_PATH, Default::default())?;
    let site = proc.create_breakpoint_site(VirtAddr(42))?;
    assert_eq!(site.addr(), VirtAddr(42));
    Ok(())
}

#[rstest]
fn test_breakpoint_site_ids_increase() -> Empty {
    let mut proc = Process::launch(LOOP_PATH, Default::default())?;

    let s1 = proc.create_breakpoint_site(VirtAddr(42))?;
    let s2 = proc.create_breakpoint_site(VirtAddr(43))?;
    let s3 = proc.create_breakpoint_site(VirtAddr(44))?;

    assert_eq!(s1.id(), 1);
    assert_eq!(s2.id(), 2);
    assert_eq!(s3.id(), 3);
    Ok(())
}

#[rstest]
fn test_breakpoint_site_duplicates() -> Empty {
    let mut proc = Process::launch(LOOP_PATH, Default::default())?;
    proc.create_breakpoint_site(VirtAddr(42))?;
    let res = proc.create_breakpoint_site(VirtAddr(42));

    assert_matches!(res, Err(DrbugError::BreakpointSiteExists(1, VirtAddr(42))));
    Ok(())
}

#[rstest]
fn test_get_site() -> Empty {
    let mut proc = Process::launch(LOOP_PATH, Default::default())?;
    let expected = proc.create_breakpoint_site(VirtAddr(42))?;

    let site = proc.breakpoint_sites().get(&1);
    assert_some_eq_x!(&site, &expected);
    Ok(())
}

#[rstest]
fn test_get_site_by_addr() -> Empty {
    let mut proc = Process::launch(LOOP_PATH, Default::default())?;
    let expected = proc.create_breakpoint_site(VirtAddr(42))?;

    let site = proc.breakpoint_sites().get_by_addr(&VirtAddr(42));
    assert_some_eq_x!(&site, &expected);
    Ok(())
}

#[rstest]
fn test_get_site_not_found() -> Empty {
    let mut proc = Process::launch(LOOP_PATH, Default::default())?;

    let site = proc.breakpoint_sites_mut().get(&1234);
    assert_none!(site);
    Ok(())
}

#[rstest]
fn test_get_site_by_addr_not_found() -> Empty {
    let mut proc = Process::launch(LOOP_PATH, Default::default())?;

    let site = proc.breakpoint_sites_mut().get_by_addr(&VirtAddr(1234));
    assert_none!(site);
    Ok(())
}

#[rstest]
fn test_remove_site() -> Empty {
    let mut proc = Process::launch(LOOP_PATH, Default::default())?;
    proc.create_breakpoint_site(VirtAddr(42))?;

    proc.breakpoint_sites_mut().remove(&1)?;
    assert!(proc.breakpoint_sites().is_empty());
    Ok(())
}

#[rstest]
fn test_break_on_address() -> Empty {
    let mut channel = Pipe::new()?;
    let opts = ProcessOptions {
        // We don't have to explicitly close the writer, because
        // taking it here drops it from the channel
        stdout: channel.take_writer().map(|w| w.into()),
        ..Default::default()
    };
    let mut proc = Process::launch(HELLO_PATH, opts)?;
    let offset = get_entry_point_offset(Path::new(HELLO_PATH));
    let load_addr = get_load_addr(proc.pid(), offset);

    proc.create_breakpoint_site(load_addr)?.enable()?;
    proc.resume()?;

    let reason = proc.wait_on_signal()?;

    assert_matches!(reason, ProcessState::Stopped { signal: Some(Signal::SIGTRAP) });
    assert_eq!(proc.get_pc()?, load_addr);

    proc.resume()?;
    let reason = proc.wait_on_signal()?;
    assert_matches!(reason, ProcessState::Exited { exit_code: 0 });

    let data = channel.read()?;
    let output = str::from_utf8(&data).unwrap();
    assert_eq!(output, "Hello, drb!\n");
    Ok(())
}

#[rstest]
fn test_remove_breakpoints() -> Empty {
    let mut proc = Process::launch(LOOP_PATH, Default::default())?;
    let site = proc.create_breakpoint_site(VirtAddr(42))?;
    let _ = proc.create_breakpoint_site(VirtAddr(43))?;

    assert_len_eq_x!(proc.breakpoint_sites(), 2);

    proc.breakpoint_sites_mut().remove(&site.id())?;
    proc.breakpoint_sites_mut().remove_by_addr(&VirtAddr(43))?;

    assert_is_empty!(proc.breakpoint_sites());
    Ok(())
}
