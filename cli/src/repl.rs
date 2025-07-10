use anyhow::anyhow;
use clap::{
    Command,
    FromArgMatches,
    Subcommand,
};
use libdrbug::prelude::*;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

#[derive(Subcommand)]
enum ReplCommand {
    #[command(about = "continue execution", visible_aliases = &["cont", "c"])]
    Continue,

    #[command(about = "display this message", visible_aliases = &["h", "?"])]
    Help,

    #[command(about = "stop debugging", visible_aliases = &["quit", "q"])]
    Exit,
}

pub fn start(mut proc: Process) -> Empty {
    let mut rl = DefaultEditor::new()?;
    let mut repl = Command::new("drb")
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .disable_version_flag(true)
        .help_template("Commands:\n{subcommands}")
        .multicall(true);
    repl = ReplCommand::augment_subcommands(repl);
    let help = repl.render_long_help();

    loop {
        let input = rl.readline("(drb) ");
        match input {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                let tokens = shlex::split(&line).ok_or(anyhow!("parse error"))?;
                let parsed_input = repl.clone().try_get_matches_from(tokens)?;
                let command = ReplCommand::from_arg_matches(&parsed_input)?;
                match command {
                    ReplCommand::Continue => {
                        proc.resume()?;
                        let status = proc.wait_on_signal()?;
                        println!("process {}: {status}", proc.pid());
                    },
                    ReplCommand::Help => println!("{help}"),
                    ReplCommand::Exit => {
                        println!("shutting down");
                        break;
                    },
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            },
            Err(err) => {
                return Err(err.into());
            },
        }
    }

    Ok(())
}
