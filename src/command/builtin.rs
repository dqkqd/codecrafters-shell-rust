use anyhow::Result;

use crate::{error::CommandError, Execute};

use super::Command;

pub enum BuiltinCommand {
    Exit(String),
    Echo(String),
    Type(String),
}

pub(super) enum ExecutableBuiltinCommand {
    Exit(i32),
    Echo(String),
    Type(Box<Command>),
}

impl TryFrom<BuiltinCommand> for ExecutableBuiltinCommand {
    type Error = CommandError;

    fn try_from(command: BuiltinCommand) -> Result<ExecutableBuiltinCommand, CommandError> {
        match command {
            BuiltinCommand::Exit(code) => Ok(ExecutableBuiltinCommand::Exit(code.parse()?)),
            BuiltinCommand::Echo(echo) => Ok(ExecutableBuiltinCommand::Echo(echo)),
            BuiltinCommand::Type(typ) => {
                let command = Command::try_from(typ)?;
                Ok(ExecutableBuiltinCommand::Type(Box::new(command)))
            }
        }
    }
}

impl Execute for ExecutableBuiltinCommand {
    fn execute(self) -> Result<()> {
        match self {
            ExecutableBuiltinCommand::Exit(code) => std::process::exit(code),
            ExecutableBuiltinCommand::Echo(echo) => println!("{}", echo),
            ExecutableBuiltinCommand::Type(typ) => match *typ {
                Command::Builtin(builtin_command) => {
                    let command_type = match builtin_command {
                        BuiltinCommand::Exit(_) => "exit",
                        BuiltinCommand::Echo(_) => "echo",
                        BuiltinCommand::Type(_) => "type",
                    };
                    println!("{} is a shell builtin", command_type);
                }
                Command::File(executable_file_command) => {
                    println!(
                        "{} is {}",
                        executable_file_command.command,
                        executable_file_command.path.display()
                    )
                }
                Command::Invalid(command) => println!("{}: not found", command),
            },
        }
        Ok(())
    }
}
