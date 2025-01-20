use anyhow::Result;
use std::{
    io::{self, Write},
    num::ParseIntError,
};
use thiserror::Error;

trait Execute {
    fn execute(self) -> Result<()>;
}

// TODO: lifetime
enum Command {
    Builtin(PlainCommand),
    Invalid(String),
}

enum PlainCommand {
    Exit(String),
    Echo(String),
    Type(String),
}

enum ExecutableCommand {
    Exit(i32),
    Echo(String),
    Type(Box<Command>),
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
        if let Some(code) = command.strip_prefix("exit") {
            Ok(Command::Builtin(PlainCommand::Exit(code.trim().into())))
        } else if let Some(echo) = command.strip_prefix("echo") {
            Ok(Command::Builtin(PlainCommand::Echo(echo.trim().into())))
        } else if let Some(typ) = command.strip_prefix("type") {
            Ok(Command::Builtin(PlainCommand::Type(typ.trim().into())))
        } else {
            Ok(Command::Invalid(command.into()))
        }
    }
}

impl TryFrom<PlainCommand> for ExecutableCommand {
    type Error = CommandError;

    fn try_from(command: PlainCommand) -> Result<ExecutableCommand, CommandError> {
        match command {
            PlainCommand::Exit(code) => Ok(ExecutableCommand::Exit(code.parse()?)),
            PlainCommand::Echo(echo) => Ok(ExecutableCommand::Echo(echo)),
            PlainCommand::Type(typ) => {
                let command = Command::try_from(typ)?;
                Ok(ExecutableCommand::Type(Box::new(command)))
            }
        }
    }
}

impl Execute for ExecutableCommand {
    fn execute(self) -> Result<()> {
        match self {
            ExecutableCommand::Exit(code) => std::process::exit(code),
            ExecutableCommand::Echo(echo) => println!("{}", echo),
            ExecutableCommand::Type(typ) => match *typ {
                Command::Builtin(plain_command) => {
                    let command_type = match plain_command {
                        PlainCommand::Exit(_) => "exit",
                        PlainCommand::Echo(_) => "echo",
                        PlainCommand::Type(_) => "type",
                    };
                    println!("{} is a shell builtin", command_type);
                }
                Command::Invalid(command) => println!("{}: not found", command),
            },
        }
        Ok(())
    }
}

impl Execute for Command {
    fn execute(self) -> Result<()> {
        match self {
            Command::Builtin(plain_command) => {
                let executable_command: ExecutableCommand = plain_command.try_into()?;
                executable_command.execute()?;
            }
            Command::Invalid(command) => println!("{}: command not found", command),
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    // Uncomment this block to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let command: Command = input.try_into().unwrap();
        command.execute()?;
    }
}
