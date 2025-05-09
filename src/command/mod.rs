use std::path::PathBuf;

use crate::io::{PErr, PIn, POut, SharedData};
use execute::Execute;
use strum::{AsRefStr, EnumIter, EnumString};

mod execute;

#[derive(Debug)]
pub(crate) enum PipedCommand {
    One(StdioCommand),
    Many(Vec<StdioCommand>),
}

impl PipedCommand {
    pub fn execute(&mut self) -> anyhow::Result<()> {
        match self {
            PipedCommand::One(stdio_command) => stdio_command.execute(),
            PipedCommand::Many(stdio_commands) => {
                for i in 0..stdio_commands.len() - 1 {
                    let shared_data = SharedData::default();

                    // first remove stdout in stdio_command
                    stdio_commands[i]
                        .stdout
                        .retain(|p| !matches!(p, POut::Std(_)));
                    // then add new shared data into stdout
                    stdio_commands[i]
                        .stdout
                        .push(POut::Shared(shared_data.clone()));

                    // add this shared data into the next command's stdin
                    stdio_commands[i + 1].stdin = PIn::Shared(shared_data.clone());
                }

                for command in stdio_commands {
                    command.execute()?;
                }

                Ok(())
            }
        }
    }
}

/// Command that supports reading from stdin and writing to stdout
#[derive(Debug)]
pub(crate) struct StdioCommand {
    stdin: PIn,
    stdout: Vec<POut>,
    stderr: Vec<PErr>,
    inner: Command,
}

impl StdioCommand {
    pub fn new(stdin: PIn, stdout: Vec<POut>, stderr: Vec<PErr>, command: Command) -> StdioCommand {
        StdioCommand {
            stdin,
            stdout,
            stderr,
            inner: command,
        }
    }

    pub fn execute(&mut self) -> anyhow::Result<()> {
        self.inner
            .execute(&mut self.stdin, &mut self.stdout, &mut self.stderr)?;
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) enum Command {
    Builtin(BuiltinCommand),
    Invalid(InvalidCommand),
    Path(PathCommand),
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct CommandArgs(pub Vec<String>);

#[derive(Debug, Default, PartialEq)]
pub(crate) struct InvalidCommand(pub String);

#[derive(Debug, Default, PartialEq)]
pub(crate) struct PathCommand {
    pub path: PathBuf,
    pub args: CommandArgs,
}

#[derive(Debug, PartialEq, EnumString, EnumIter, AsRefStr)]
pub(crate) enum BuiltinCommand {
    #[strum(serialize = "exit")]
    Exit(CommandArgs),
    #[strum(serialize = "echo")]
    Echo(CommandArgs),
    #[strum(serialize = "type")]
    Type(CommandArgs),
    #[strum(serialize = "pwd")]
    Pwd,
    #[strum(serialize = "cd")]
    Cd(CommandArgs),
}

impl BuiltinCommand {
    pub fn with_args(self, args: CommandArgs) -> BuiltinCommand {
        match self {
            BuiltinCommand::Exit(_) => BuiltinCommand::Exit(args),
            BuiltinCommand::Echo(_) => BuiltinCommand::Echo(args),
            BuiltinCommand::Type(_) => BuiltinCommand::Type(args),
            BuiltinCommand::Pwd => BuiltinCommand::Pwd,
            BuiltinCommand::Cd(_) => BuiltinCommand::Cd(args),
        }
    }
}
