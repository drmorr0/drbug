use super::*;
use crate::DrbugError;
use crate::breakpoint::Breakable;

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
    assert_eq!(&expected, site);
}

#[rstest]
fn test_get_site_mut() {
    let mut proc = Process::launch(LOOP_PATH, Default::default()).unwrap();
    let expected = proc.create_breakpoint_site(VirtAddr(42)).unwrap();

    let site = proc.breakpoint_sites_mut().get_mut(&1).unwrap();
    assert_eq!(&expected, site);
}

#[rstest]
fn test_get_site_by_addr() {
    let mut proc = Process::launch(LOOP_PATH, Default::default()).unwrap();
    let expected = proc.create_breakpoint_site(VirtAddr(42)).unwrap();

    let site = proc.breakpoint_sites().get_by_addr(&VirtAddr(42)).unwrap();
    assert_eq!(&expected, site);
}

#[rstest]
fn test_get_site_by_addr_mut() {
    let mut proc = Process::launch(LOOP_PATH, Default::default()).unwrap();
    let expected = proc.create_breakpoint_site(VirtAddr(42)).unwrap();

    let site = proc.breakpoint_sites_mut().get_by_addr_mut(&VirtAddr(42)).unwrap();
    assert_eq!(&expected, site);
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

    proc.breakpoint_sites_mut().remove(&1);
    assert!(proc.breakpoint_sites().is_empty());
}
