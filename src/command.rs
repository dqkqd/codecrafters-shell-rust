mod executable;
mod plain;

use anyhow::Result;
use executable::ExecutableCommand;
use plain::PlainCommand;

use crate::{error::CommandError, Execute};

// TODO: lifetime
pub enum Command {
    Builtin(PlainCommand),
    Invalid(String),
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
