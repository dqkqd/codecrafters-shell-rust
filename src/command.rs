mod builtin;
mod executable_file;

use std::io::{self, Write};

use anyhow::Result;
use builtin::{BuiltinCommand, ExecutableBuiltinCommand};
use executable_file::ExecutableFileCommand;

use crate::{error::CommandError, Execute};

// TODO: lifetime
pub enum Command {
    Builtin(BuiltinCommand),
    File(ExecutableFileCommand),
    Invalid(String),
}

impl TryFrom<String> for Command {
    type Error = CommandError;

    fn try_from(command: String) -> Result<Command, CommandError> {
        let command = command.trim();
        if let Some(code) = command.strip_prefix("exit") {
            Ok(Command::Builtin(BuiltinCommand::Exit(code.trim().into())))
        } else if let Some(echo) = command.strip_prefix("echo") {
            Ok(Command::Builtin(BuiltinCommand::Echo(echo.trim().into())))
        } else if command.strip_prefix("pwd").is_some() {
            Ok(Command::Builtin(BuiltinCommand::Pwd))
        } else if let Some(typ) = command.strip_prefix("type") {
            Ok(Command::Builtin(BuiltinCommand::Type(typ.trim().into())))
        } else if let Ok(executable_file_command) = ExecutableFileCommand::try_from(command) {
            Ok(Command::File(executable_file_command))
        } else {
            Ok(Command::Invalid(command.into()))
        }
    }
}

impl Execute for Command {
    fn execute(self) -> Result<()> {
        match self {
            Command::Builtin(builtin_command) => {
                let executable_command: ExecutableBuiltinCommand = builtin_command.try_into()?;
                executable_command.execute()?;
            }
            Command::File(executable_file_command) => {
                let output = std::process::Command::new(executable_file_command.command)
                    .args(executable_file_command.args)
                    .output()?;
                io::stdout().write_all(&output.stdout)?;
                io::stderr().write_all(&output.stderr)?;
            }
            Command::Invalid(command) => println!("{}: command not found", command),
        }
        Ok(())
    }
}
