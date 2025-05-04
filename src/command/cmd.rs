use super::io::{PErr, POut};

pub(super) enum InternalCommand {
    Builtin(Builtin),
    Invalid(String),
}

pub(super) enum Builtin {
    Exit(i32),
    Echo(String),
    Type(String),
}

pub(crate) struct Command {
    stdout: POut,
    #[allow(unused)]
    stderr: PErr,
    command: InternalCommand,
}

impl Command {
    pub(super) fn new(stdout: POut, stderr: PErr, command: InternalCommand) -> Command {
        Command {
            stdout,
            stderr,
            command,
        }
    }

    pub fn execute(&mut self) -> anyhow::Result<()> {
        match self.command {
            InternalCommand::Builtin(Builtin::Exit(code)) => std::process::exit(code),
            InternalCommand::Builtin(Builtin::Echo(ref s)) => {
                self.stdout.write_all_and_flush(&format!("{s}\n"))?;
            }
            InternalCommand::Builtin(Builtin::Type(ref command)) => match command.trim() {
                command @ ("echo" | "exit" | "type") => {
                    self.stdout
                        .write_all_and_flush(&format!("{command} is a shell builtin\n"))?;
                }
                command => {
                    self.stdout
                        .write_all_and_flush(&format!("{command}: not found\n"))?;
                }
            },
            InternalCommand::Invalid(ref command) => {
                self.stdout
                    .write_all_and_flush(&format!("{command}: command not found\n"))?;
            }
        }
        Ok(())
    }
}
