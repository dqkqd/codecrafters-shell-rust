use anyhow::Result;

use crate::{error::CommandError, Execute};

use super::{plain::PlainCommand, Command};

pub(super) enum ExecutableCommand {
    Exit(i32),
    Echo(String),
    Type(Box<Command>),
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
