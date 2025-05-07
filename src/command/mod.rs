use std::path::PathBuf;

use crate::io::{PErr, PIn, POut};
use execute::Execute;
use strum::EnumString;

mod execute;

#[derive(Debug)]
pub(crate) struct PipedCommand {
    stdin: Vec<PIn>,
    stdout: Vec<POut>,
    stderr: Vec<PErr>,
    inner: Command,
}

impl PipedCommand {
    pub fn new(
        stdin: Vec<PIn>,
        stdout: Vec<POut>,
        stderr: Vec<PErr>,
        command: Command,
    ) -> PipedCommand {
        PipedCommand {
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
pub(crate) struct ProgramArgs(pub Vec<String>);

#[derive(Debug, Default, PartialEq)]
pub(crate) struct InvalidCommand(pub String);

#[derive(Debug, Default, PartialEq)]
pub(crate) struct PathCommand {
    pub path: PathBuf,
    pub args: ProgramArgs,
}

#[derive(Debug, PartialEq, EnumString)]
pub(crate) enum BuiltinCommand {
    #[strum(serialize = "exit")]
    Exit(ProgramArgs),
    #[strum(serialize = "echo")]
    Echo(ProgramArgs),
    #[strum(serialize = "type")]
    Type(ProgramArgs),
    #[strum(serialize = "pwd")]
    Pwd,
    #[strum(serialize = "cd")]
    Cd(ProgramArgs),
}

impl BuiltinCommand {
    pub fn with_args(self, args: ProgramArgs) -> BuiltinCommand {
        match self {
            BuiltinCommand::Exit(_) => BuiltinCommand::Exit(args),
            BuiltinCommand::Echo(_) => BuiltinCommand::Echo(args),
            BuiltinCommand::Type(_) => BuiltinCommand::Type(args),
            BuiltinCommand::Pwd => BuiltinCommand::Pwd,
            BuiltinCommand::Cd(_) => BuiltinCommand::Cd(args),
        }
    }
}
