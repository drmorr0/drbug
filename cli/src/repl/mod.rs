mod commands;
mod register;

use anyhow::anyhow;
use clap::Parser;
use libdrbug::prelude::*;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use self::commands::*;
use crate::Empty;

pub struct Repl {
    proc: Process,
    rl: DefaultEditor,
    running: bool,
}

impl Repl {
    pub fn new(proc: Process) -> anyhow::Result<Repl> {
        Ok(Repl { proc, rl: DefaultEditor::new()?, running: true })
    }

    pub fn start(&mut self) -> Empty {
        while self.running {
            let input = self.rl.readline("(drb) ");
            match input {
                Ok(line) => {
                    if let Err(err) = self.handle_line(line) {
                        println!("{err}");
                    }
                },
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C; shutting down");
                    self.running = false;
                },
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D; shutting down");
                    self.running = false;
                },
                Err(err) => {
                    println!("{err}");
                },
            }
        }

        Ok(())
    }

    fn handle_line(&mut self, line: String) -> Empty {
        self.rl.add_history_entry(line.as_str())?;
        let tokens = shlex::split(&line).ok_or(anyhow!("parse error"))?;
        let root = DrbRootCommand::try_parse_from(tokens)?;

        match &root.command {
            ReplCommand::Continue => {
                self.proc.resume()?;
                let status = self.proc.wait_on_signal()?;
                let pc = self.proc.get_pc()?;
                println!("process {}: {status} at {pc}", self.proc.pid());
            },
            ReplCommand::Register(cmd) => register::handle(cmd, &mut self.proc)?,
            ReplCommand::Quit => {
                self.running = false;
            },
        }

        Ok(())
    }
}
