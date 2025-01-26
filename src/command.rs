mod builtin;
mod exec_file;

use anyhow::Result;
use builtin::{BuiltinCmd, ExecBuiltinCmd};
use exec_file::ExecFileCmd;

use crate::{
    execute::{Execute, ExecutedOutput},
    token::{parse_tokens, ValueToken},
};

// TODO: lifetime
pub(crate) enum Cmd {
    Builtin(BuiltinCmd),
    ExecFile(ExecFileCmd),
    Invalid(String),
}

impl Cmd {
    pub fn from_value_tokens(mut values: Vec<ValueToken>) -> Result<Cmd> {
        let command = values.remove(0);

        let args: Vec<String> = values
            .into_iter()
            .filter_map(|v| v.try_into().ok())
            .collect();

        match command.0.as_slice() {
            b"exit" => Ok(Cmd::Builtin(BuiltinCmd::Exit(
                args.first().cloned().unwrap_or_default(),
            ))),
            b"pwd" => Ok(Cmd::Builtin(BuiltinCmd::Pwd)),
            b"echo" => Ok(Cmd::Builtin(BuiltinCmd::Echo(args.join(" ")))),
            b"cd" => Ok(Cmd::Builtin(BuiltinCmd::Cd(args.join(" ")))),
            b"type" => Ok(Cmd::Builtin(BuiltinCmd::Type(args.join(" ")))),
            _ => {
                let command: String = command.try_into()?;
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
