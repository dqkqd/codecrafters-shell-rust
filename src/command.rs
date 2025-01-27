mod builtin;
mod exec_file;

use anyhow::Result;
use builtin::{BuiltinCmd, ExecBuiltinCmd};
use exec_file::ExecFileCmd;

use crate::parser::ValueToken;

// TODO: lifetime
pub(crate) enum Cmd {
    Builtin(BuiltinCmd),
    ExecFile(ExecFileCmd),
    Invalid(String),
}

pub(crate) trait Execute {
    fn execute(self) -> Result<ExecutedOutput>;
}

#[derive(Default)]
pub(crate) struct ExecutedOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl Cmd {
    pub fn from_value_tokens(mut values: Vec<ValueToken>) -> Result<Cmd> {
        let command = values.remove(0);

        let args: Vec<String> = values.into_iter().map(|v| v.0).collect();

        match command.0.as_str() {
            "exit" => Ok(Cmd::Builtin(BuiltinCmd::Exit(
                args.first().cloned().unwrap_or_default(),
            ))),
            "pwd" => Ok(Cmd::Builtin(BuiltinCmd::Pwd)),
            "echo" => Ok(Cmd::Builtin(BuiltinCmd::Echo(args.join(" ")))),
            "cd" => Ok(Cmd::Builtin(BuiltinCmd::Cd(args.join(" ")))),
            "type" => Ok(Cmd::Builtin(BuiltinCmd::Type(args.join(" ")))),
            _ => {
                let command: String = command.0;
                if let Ok(exec_file_cmd) = ExecFileCmd::new(command.clone(), args) {
                    Ok(Cmd::ExecFile(exec_file_cmd))
                } else {
                    Ok(Cmd::Invalid(command))
                }
            }
        }
    }
}

impl Execute for Cmd {
    fn execute(self) -> Result<ExecutedOutput> {
        let output = match self {
            Cmd::Builtin(builtin_command) => {
                let executable_command: ExecBuiltinCmd = builtin_command.into_exec()?;
                executable_command.execute()?
            }
            Cmd::ExecFile(executable_file_command) => executable_file_command.execute()?,
            Cmd::Invalid(command) => {
                ExecutedOutput::new().with_stdout(format!("{}: command not found", command))
            }
        };

        Ok(output)
    }
}

impl ExecutedOutput {
    pub fn new() -> ExecutedOutput {
        ExecutedOutput::default()
    }

    pub fn with_stdout<S: AsRef<[u8]>>(mut self, s: S) -> ExecutedOutput {
        self.stdout = s.as_ref().into();
        self
    }

    pub fn with_stderr<S: AsRef<[u8]>>(mut self, s: S) -> ExecutedOutput {
        self.stderr = s.as_ref().into();
        self
    }
}
