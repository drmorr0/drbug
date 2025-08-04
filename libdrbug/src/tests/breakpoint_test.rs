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
fn test_create_breakpoint_site() {
    let mut proc = Process::launch(LOOP_PATH, Default::default()).unwrap();
    let site = proc.create_breakpoint_site(VirtAddr(42)).unwrap();
    assert_eq!(site.addr(), VirtAddr(42));
}

#[rstest]
fn test_breakpoint_site_ids_increase() {
    let mut proc = Process::launch(LOOP_PATH, Default::default()).unwrap();

    let s1 = proc.create_breakpoint_site(VirtAddr(42)).unwrap();
    let s2 = proc.create_breakpoint_site(VirtAddr(43)).unwrap();
    let s3 = proc.create_breakpoint_site(VirtAddr(44)).unwrap();

    assert_eq!(s1.id(), 1);
    assert_eq!(s2.id(), 2);
    assert_eq!(s3.id(), 3);
}

#[rstest]
fn test_breakpoint_site_duplicates() {
    let mut proc = Process::launch(LOOP_PATH, Default::default()).unwrap();
    proc.create_breakpoint_site(VirtAddr(42)).unwrap();
    let res = proc.create_breakpoint_site(VirtAddr(42));

    assert_matches!(res, Err(DrbugError::BreakpointSiteExists(1, VirtAddr(42))));
}

#[rstest]
fn test_get_site() {
    let mut proc = Process::launch(LOOP_PATH, Default::default()).unwrap();
    let expected = proc.create_breakpoint_site(VirtAddr(42)).unwrap();

    let site = proc.breakpoint_sites().get(&1).unwrap();
    assert_eq!(expected, site);
}

#[rstest]
fn test_get_site_by_addr() {
    let mut proc = Process::launch(LOOP_PATH, Default::default()).unwrap();
    let expected = proc.create_breakpoint_site(VirtAddr(42)).unwrap();

    let site = proc.breakpoint_sites().get_by_addr(&VirtAddr(42)).unwrap();
    assert_eq!(expected, site);
}

#[rstest]
fn test_get_site_not_found() {
    let mut proc = Process::launch(LOOP_PATH, Default::default()).unwrap();

    let site = proc.breakpoint_sites_mut().get(&1234);
    assert_none!(site);
}

#[rstest]
fn test_get_site_by_addr_not_found() {
    let mut proc = Process::launch(LOOP_PATH, Default::default()).unwrap();

    let site = proc.breakpoint_sites_mut().get_by_addr(&VirtAddr(1234));
    assert_none!(site);
}

#[rstest]
fn test_remove_site() {
    let mut proc = Process::launch(LOOP_PATH, Default::default()).unwrap();
    proc.create_breakpoint_site(VirtAddr(42)).unwrap();

    proc.breakpoint_sites_mut().remove(&1).unwrap();
    assert!(proc.breakpoint_sites().is_empty());
}

#[rstest]
fn test_break_on_address() {
    let mut channel = Pipe::new().unwrap();
    let opts = ProcessOptions {
        stdout: channel.take_writer().map(|w| w.into()),
        ..Default::default()
    };
    let mut proc = Process::launch(HELLO_PATH, opts).unwrap();
    let offset = get_entry_point_offset(Path::new(HELLO_PATH));
    let load_addr = get_load_addr(proc.pid(), offset);

    proc.create_breakpoint_site(load_addr).unwrap().enable().unwrap();
    proc.resume().unwrap();

    let reason = proc.wait_on_signal().unwrap();

    assert_matches!(reason, ProcessState::Stopped { signal: Some(Signal::SIGTRAP) });
    assert_eq!(proc.get_pc().unwrap(), load_addr);

    proc.resume().unwrap();
    let reason = proc.wait_on_signal().unwrap();
    assert_matches!(reason, ProcessState::Exited { exit_code: 0 });

    let data = channel.read().unwrap();
    let output = str::from_utf8(&data).unwrap();
    assert_eq!(output, "Hello, drb!\n");
}

#[rstest]
fn test_remove_breakpoints() {
    let mut proc = Process::launch(LOOP_PATH, Default::default()).unwrap();
    let site = proc.create_breakpoint_site(VirtAddr(42)).unwrap();
    let _ = proc.create_breakpoint_site(VirtAddr(43)).unwrap();

    assert_len_eq_x!(proc.breakpoint_sites(), 2);

    proc.breakpoint_sites_mut().remove(&site.id()).unwrap();
    proc.breakpoint_sites_mut().remove_by_addr(&VirtAddr(43)).unwrap();

    assert_is_empty!(proc.breakpoint_sites());
}
