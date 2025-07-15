mod commands;
mod register;

use anyhow::anyhow;
use clap::Parser;
use libdrbug::prelude::*;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use self::commands::*;

pub fn start(mut proc: Process) -> Empty {
    let mut rl = DefaultEditor::new()?;

    loop {
        let input = rl.readline("(drb) ");
        match input {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                let tokens = shlex::split(&line).ok_or(anyhow!("parse error"))?;
                let command = match DrbRootCommand::try_parse_from(tokens) {
                    Ok(repl) => repl.command,
                    Err(e) => {
                        println!("{e}");
                        continue;
                    },
                };

                match &command {
                    ReplCommand::Continue => {
                        proc.resume()?;
                        let status = proc.wait_on_signal()?;
                        println!("process {}: {status}", proc.pid());
                    },
                    ReplCommand::Register(cmd) => register::handle(cmd, &mut proc)?,
                    ReplCommand::Quit => {
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
