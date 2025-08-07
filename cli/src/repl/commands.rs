use clap::{
    Parser,
    Subcommand,
};

use super::breakpoint::*;
use super::memory::*;
use super::register::*;

#[derive(Parser)]
#[command(
    disable_help_flag = true,
    disable_version_flag = true,
    help_template = "type 'command help' for additional information\n\n{all-args}",
    multicall = true
)]
pub(super) struct DrbRootCommand {
    #[command(subcommand)]
    pub(super) command: ReplCommand,
}

#[derive(Subcommand)]
pub(super) enum ReplCommand {
    #[command(subcommand, about = "manage breakpoints", visible_aliases = &["b", "br", "bp", "break"])]
    Breakpoint(BreakpointCommand),

    #[command(about = "continue execution", visible_aliases = &["cont", "c"])]
    Continue,

    #[command(subcommand, about = "read and write to memory locations", visible_aliases = &["mem"])]
    Memory(MemoryCommand),

    #[command(subcommand, about = "interact with registers", visible_aliases = &["reg"])]
    Register(RegisterCommand),

    #[command(about ="step over a single instruction", visible_aliases = &["s", "st"])]
    Step,

    #[command(about = "stop debugging", visible_aliases = &["exit", "q"])]
    Quit,
}
