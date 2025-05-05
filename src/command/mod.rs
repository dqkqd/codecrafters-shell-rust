use std::path::PathBuf;

use execute::Execute;
use io::{PErr, PIn, POut};
use parse::{parse_command, ParseInput};
use strum::EnumString;

mod execute;
mod io;
mod parse;

pub(crate) struct Command {
    stdin: PIn,
    stdout: POut,
    stderr: PErr,
    inner: InternalCommand,
}

impl Command {
    fn new(stdin: PIn, stdout: POut, stderr: PErr, command: InternalCommand) -> Command {
        Command {
            stdin,
            stdout,
            stderr,
            inner: command,
        }
    }

    pub fn parse(input: ParseInput) -> Command {
        parse_command(input)
    }

    pub fn execute(&mut self) -> anyhow::Result<()> {
        self.inner
            .execute(&mut self.stdin, &mut self.stdout, &mut self.stderr)?;
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
struct Args(pub Vec<String>);

#[derive(Debug, Default, PartialEq)]
struct InvalidCommand(pub String);

#[derive(Debug, Default, PartialEq)]
struct PathCommand {
    pub path: PathBuf,
    pub args: Args,
}

#[derive(Debug, PartialEq, EnumString)]
enum BuiltinCommand {
    #[strum(serialize = "exit")]
    Exit(Args),
    #[strum(serialize = "echo")]
    Echo(Args),
    #[strum(serialize = "type")]
    Type(Args),
    #[strum(serialize = "pwd")]
    Pwd,
    #[strum(serialize = "cd")]
    Cd(Args),
}

impl BuiltinCommand {
    fn with_args(self, args: Args) -> BuiltinCommand {
        match self {
            BuiltinCommand::Exit(_) => BuiltinCommand::Exit(args),
            BuiltinCommand::Echo(_) => BuiltinCommand::Echo(args),
            BuiltinCommand::Type(_) => BuiltinCommand::Type(args),
            BuiltinCommand::Pwd => BuiltinCommand::Pwd,
            BuiltinCommand::Cd(_) => BuiltinCommand::Cd(args),
        }
    }
}
