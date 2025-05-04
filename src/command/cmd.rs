use super::io::{PErr, POut};

pub(crate) struct CommandWithPipe {
    stdout: POut,
    #[allow(unused)]
    stderr: PErr,
    command: Command,
}

impl CommandWithPipe {
    pub fn new(stdout: POut, stderr: PErr, command: Command) -> CommandWithPipe {
        CommandWithPipe {
            stdout,
            stderr,
            command,
        }
    }

    pub fn execute(&mut self) -> anyhow::Result<()> {
        match self.command {
            Command::Builtin(BuiltinCommand::ExitCommand(code)) => std::process::exit(code),
            Command::InvalidCommand(ref command) => {
                self.stdout
                    .write_all_and_flush(&format!("{command}: command not found\n"))?;
            }
        }
        Ok(())
    }
}

pub(crate) enum Command {
    Builtin(BuiltinCommand),
    InvalidCommand(String),
}

pub(crate) enum BuiltinCommand {
    ExitCommand(i32),
}
