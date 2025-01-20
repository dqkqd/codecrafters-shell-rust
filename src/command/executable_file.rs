use std::path::PathBuf;

use crate::{error::CommandError, Execute};

pub struct ExecutableFileCommand {
    pub command: String,
    pub path: PathBuf,
    pub args: Vec<String>,
}

impl Execute for ExecutableFileCommand {
    fn execute(self) -> anyhow::Result<()> {
        todo!()
    }
}

impl TryFrom<&str> for ExecutableFileCommand {
    type Error = CommandError;

    fn try_from(command: &str) -> Result<ExecutableFileCommand, CommandError> {
        let path = std::env::var("PATH")?;
        let (command, args) = command.split_once(' ').unwrap_or((command, ""));

        let command_path = path
            .split(":")
            .map(|path| PathBuf::from(path).join(command))
            .find(|path| path.is_file())
            .ok_or_else(|| {
                CommandError::MissingCommand("missing executable file command".into())
            })?;

        let args = args
            .split_whitespace()
            .map(|arg| arg.trim().to_string())
            .collect();

        Ok(ExecutableFileCommand {
            command: command.into(),
            path: command_path,
            args,
        })
    }
}
