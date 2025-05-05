use std::path::PathBuf;

use execute::Execute;
use io::{PErr, PIn, POut};
use parse::{parse_command, ParseInput};
use strum::EnumString;

mod execute;
mod io;
mod parse;

pub(crate) struct Command {
    stdin: Vec<PIn>,
    stdout: Vec<POut>,
    stderr: Vec<PErr>,
    inner: InternalCommand,
}

impl Command {
    fn new(
        stdin: Vec<PIn>,
        stdout: Vec<POut>,
        stderr: Vec<PErr>,
        command: InternalCommand,
    ) -> Command {
        Command {
            stdin,
            stdout,
            stderr,
            inner: command,
        }
    }

    pub fn parse(input: ParseInput) -> anyhow::Result<Command> {
        parse_command(input)
    }

    pub fn execute(&mut self) -> anyhow::Result<()> {
        self.inner
            .execute(self.stdin.as_mut(), &mut self.stdout, &mut self.stderr)?;
        Ok(())
    }
}

#[derive(Debug)]
enum InternalCommand {
    Builtin(BuiltinCommand),
    Invalid(InvalidCommand),
    Path(PathCommand),
}

#[derive(Debug, Default, PartialEq)]
struct ProgramArgs(pub Vec<String>);

#[derive(Debug, Default, PartialEq)]
struct InvalidCommand(pub String);

#[derive(Debug, Default, PartialEq)]
struct PathCommand {
    pub path: PathBuf,
    pub args: ProgramArgs,
}

#[derive(Debug, PartialEq, EnumString)]
enum BuiltinCommand {
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
    fn with_args(self, args: ProgramArgs) -> BuiltinCommand {
        match self {
            BuiltinCommand::Exit(_) => BuiltinCommand::Exit(args),
            BuiltinCommand::Echo(_) => BuiltinCommand::Echo(args),
            BuiltinCommand::Type(_) => BuiltinCommand::Type(args),
            BuiltinCommand::Pwd => BuiltinCommand::Pwd,
            BuiltinCommand::Cd(_) => BuiltinCommand::Cd(args),
        }
    }
}
