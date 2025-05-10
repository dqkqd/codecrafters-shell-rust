use std::path::PathBuf;

use crate::io::{PErr, PIn, POut};
use anyhow::Result;
use execute::{Execute, ExecutedOutput};
use strum::{AsRefStr, EnumIter, EnumString};

mod execute;

#[derive(Debug)]
pub(crate) struct PipeCommands {
    pub commands: Vec<StdioCommand>,
}

impl PipeCommands {
    pub fn execute(mut self) -> Result<()> {
        // set up pipe
        for i in 0..self.commands.len() - 1 {
            let (tx, rx) = std::sync::mpsc::channel();

            // first remove stdout in stdio_command
            self.commands[i]
                .stdout
                .retain(|p| !matches!(p, POut::Std(_)));
            // then add new shared data into stdout
            self.commands[i].stdout.push(POut::Pipe(tx));

            // add this shared data into the next command's stdin
            self.commands[i + 1].stdin = PIn::Pipe(rx);
        }

        let output: Result<Vec<ExecutedOutput>> = self
            .commands
            .into_iter()
            .map(|command| command.execute())
            .collect();
        let mut outputs = output?;

        // only wait for the last execution, then kill others
        outputs.pop().and_then(|out| out.wait().ok());
        for out in outputs {
            out.kill()?;
        }

        Ok(())
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

    pub fn execute(mut self) -> Result<ExecutedOutput> {
        self.inner.execute(self.stdin, self.stdout, self.stderr)
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
