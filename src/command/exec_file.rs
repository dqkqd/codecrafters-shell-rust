use std::path::PathBuf;

use crate::{error::CmdError, Execute};

pub struct ExecFileCmd {
    pub command: String,
    pub path: PathBuf,
    pub args: Vec<String>,
}

impl Execute for ExecFileCmd {
    fn execute(self) -> anyhow::Result<()> {
        todo!()
    }
}

impl TryFrom<&str> for ExecFileCmd {
    type Error = CmdError;

    fn try_from(command: &str) -> Result<ExecFileCmd, CmdError> {
        let path = std::env::var("PATH")?;
        let (command, args) = command.split_once(' ').unwrap_or((command, ""));

        let command_path = path
            .split(":")
            .map(|path| PathBuf::from(path).join(command))
            .find(|path| path.is_file())
            .ok_or_else(|| CmdError::MissingCmd("missing executable file command".into()))?;

        let args = args
            .split_whitespace()
            .map(|arg| arg.trim().to_string())
            .collect();

        Ok(ExecFileCmd {
            command: command.into(),
            path: command_path,
            args,
        })
    }
}
