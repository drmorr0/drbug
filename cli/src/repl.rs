use libdrbug::prelude::*;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

pub fn start() -> Empty {
    let mut rl = DefaultEditor::new()?;

    loop {
        let input = rl.readline("(drb) ");
        match input {
            Ok(_line) => {},
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
