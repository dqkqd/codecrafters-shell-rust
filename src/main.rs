use std::{
    io::{self, Write},
    num::ParseIntError,
};
use thiserror::Error;

// TODO: lifetime
enum Command {
    Exit(i32),
    Echo(String),
    Invalid(String),
}

#[derive(Error, Debug)]
enum CommandError {
    #[error("parse int error")]
    ParseIntError(#[from] ParseIntError),
}

impl TryFrom<String> for Command {
    type Error = CommandError;

    fn try_from(command: String) -> Result<Command, CommandError> {
        let command = command.trim();
        if let Some(code) = command.strip_prefix("exit ") {
            let code = code.parse()?;
            Ok(Command::Exit(code))
        } else if let Some(echo) = command.strip_prefix("echo ") {
            Ok(Command::Echo(echo.into()))
        } else {
            Ok(Command::Invalid(command.into()))
        }
    }
}

impl Command {
    fn execute(self) {
        match self {
            Command::Exit(code) => std::process::exit(code),
            Command::Echo(s) => println!("{}", s),
            Command::Invalid(s) => println!("{}: command not found", s),
        }
    }
}

fn main() {
    // Uncomment this block to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let cmd: Command = input.try_into().unwrap();
        cmd.execute();
    }
}
