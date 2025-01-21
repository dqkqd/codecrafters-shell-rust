mod builtin;
mod exec_file;

use std::io::{self, Write};

use anyhow::Result;
use builtin::{BuiltinCmd, ExecBuiltinCmd};
use exec_file::ExecFileCmd;

use crate::{error::CmdError, Execute};

// TODO: lifetime
pub enum Cmd {
    Builtin(BuiltinCmd),
    ExecFileCmd(ExecFileCmd),
    Invalid(String),
}

impl TryFrom<String> for Cmd {
    type Error = CmdError;

    fn try_from(command: String) -> Result<Cmd, CmdError> {
        let command = command.trim();
        if let Some(code) = command.strip_prefix("exit") {
            Ok(Cmd::Builtin(BuiltinCmd::Exit(code.trim().into())))
        } else if let Some(echo) = command.strip_prefix("echo") {
            Ok(Cmd::Builtin(BuiltinCmd::Echo(echo.trim().into())))
        } else if command.strip_prefix("pwd").is_some() {
            Ok(Cmd::Builtin(BuiltinCmd::Pwd))
        } else if let Some(cd) = command.strip_prefix("cd") {
            Ok(Cmd::Builtin(BuiltinCmd::Cd(cd.trim().into())))
        } else if let Some(typ) = command.strip_prefix("type") {
            Ok(Cmd::Builtin(BuiltinCmd::Type(typ.trim().into())))
        } else if let Ok(executable_file_command) = ExecFileCmd::try_from(command) {
            Ok(Cmd::ExecFileCmd(executable_file_command))
        } else {
            Ok(Cmd::Invalid(command.into()))
        }
    }
}

impl Execute for Cmd {
    fn execute(self) -> Result<()> {
        match self {
            Cmd::Builtin(builtin_command) => {
                let executable_command: ExecBuiltinCmd = builtin_command.try_into()?;
                executable_command.execute()?;
            }
            Cmd::ExecFileCmd(executable_file_command) => {
                let output = std::process::Command::new(executable_file_command.command)
                    .args(executable_file_command.args)
                    .output()?;
                io::stdout().write_all(&output.stdout)?;
                io::stderr().write_all(&output.stderr)?;
            }
            Cmd::Invalid(command) => println!("{}: command not found", command),
        }
        Ok(())
    }
}
