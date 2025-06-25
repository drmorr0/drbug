use libdrbug::prelude::*;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

pub fn start(mut proc: Process) -> Empty {
    let mut rl = DefaultEditor::new()?;

    loop {
        let input = rl.readline("(drb) ");
        match input {
            Ok(line) => {
                let mut tokens = line.split(' ');
                let command = tokens.next().unwrap();
                match command {
                    cmd if "continue".starts_with(cmd) => {
                        proc.resume()?;
                        let status = proc.wait_on_signal()?;
                        println!("Status: {status:?}");
                    },
                    _ => println!("unknown command: {command}"),
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
