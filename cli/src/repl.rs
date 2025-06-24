use libdrbug::prelude::*;
use nix::sys::ptrace;
use nix::sys::wait::waitpid;
use nix::unistd::Pid;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

pub fn start(pid: Pid) -> Empty {
    let mut rl = DefaultEditor::new()?;

    loop {
        let input = rl.readline("(drb) ");
        match input {
            Ok(line) => {
                let mut tokens = line.split(' ');
                let command = tokens.next().unwrap();
                match command {
                    cmd if "continue".starts_with(cmd) => resume(pid)?,
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

fn resume(pid: Pid) -> Empty {
    ptrace::cont(pid, None)?;
    waitpid(pid, None)?;
    Ok(())
}
