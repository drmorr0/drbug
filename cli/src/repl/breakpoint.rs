use clap::{
    Args,
    Subcommand,
};
use libdrbug::prelude::*;

use crate::Empty;

#[derive(Subcommand)]
pub(super) enum BreakpointCommand {
    #[command(about = "delete a breakpoint", visible_aliases = &["del", "rm"])]
    Delete(BpArgs),

    #[command(about = "disable a breakpoint", visible_aliases = &["dis"])]
    Disable(BpArgs),

    #[command(about = "enable a breakpoint", visible_aliases = &["en"])]
    Enable(BpArgs),

    #[command(about = "list all breakpoints", visible_aliases = &["l", "ls"])]
    List,

    #[command(about = "set a breakpoint")]
    Set(BpSetArgs),
}

#[derive(Args)]
pub(super) struct BpArgs {
    #[arg(long_help = "id of breakpoint to operate on")]
    id: usize,
}

#[derive(Args)]
pub(super) struct BpSetArgs {
    #[arg(long_help = "memory address to break on")]
    location: VirtAddr,
}


pub(super) fn handle(command: &BreakpointCommand, proc: &mut Process) -> Empty {
    match command {
        BreakpointCommand::Delete(args) => handle_delete(proc, args.id),
        BreakpointCommand::Disable(args) => handle_disable(proc, args.id),
        BreakpointCommand::Enable(args) => handle_enable(proc, args.id),
        BreakpointCommand::List => {
            handle_list(proc);
            Ok(())
        },
        BreakpointCommand::Set(args) => handle_set(proc, args.location),
    }
}

fn handle_delete(proc: &mut Process, id: usize) -> Empty {
    if proc.breakpoint_sites().get(&id).is_none() {
        println!("breakpoint {id} not found");
    }
    proc.breakpoint_sites_mut().remove(&id)?;
    println!("breakpoint {id} deleted");
    Ok(())
}

fn handle_disable(proc: &mut Process, id: usize) -> Empty {
    if let Some(mut site) = proc.breakpoint_sites_mut().get(&id) {
        site.disable()?;
        println!("breakpoint {id} at {:#x} disabled", site.addr());
    } else {
        println!("breakpoint {id} not found");
    }
    Ok(())
}

fn handle_enable(proc: &mut Process, id: usize) -> Empty {
    if let Some(mut site) = proc.breakpoint_sites_mut().get(&id) {
        site.enable()?;
        println!("breakpoint {id} at {:#x} enabled", site.addr());
    } else {
        println!("breakpoint {id} not found");
    }
    Ok(())
}

fn handle_list(proc: &Process) {
    let sites = proc.breakpoint_sites();
    if sites.is_empty() {
        println!("no breakpoints set");
    } else {
        println!("current breakpoints:");
        for (id, site) in sites.iter() {
            println!("{id}: address = {:#x}, {}", site.addr(), if site.enabled() { "enabled" } else { "disabled" });
        }
    }
}

fn handle_set(proc: &mut Process, loc: VirtAddr) -> Empty {
    proc.create_breakpoint_site(loc)?.enable()?;
    Ok(())
}
