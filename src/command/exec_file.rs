use std::{
    io::{self, Write},
    path::PathBuf,
};

use anyhow::Result;

use crate::{error::CmdError, Execute};

pub struct ExecFileCmd {
    pub command: String,
    pub path: PathBuf,
    pub args: Vec<String>,
}

impl ExecFileCmd {
    pub fn new(command: String, args: Vec<String>) -> Result<ExecFileCmd, CmdError> {
        let path = std::env::var("PATH")?;
        let path = path
            .split(":")
            .map(|path| PathBuf::from(path).join(&command))
            .find(|path| path.is_file())
            .ok_or_else(|| CmdError::MissingCmd("missing executable file command".into()))?;

        Ok(ExecFileCmd {
            command,
            path,
            args,
        })
    }
}

impl Execute for ExecFileCmd {
    fn execute(self) -> Result<()> {
        let output = std::process::Command::new(self.command)
            .args(self.args)
            .output()?;
        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)?;
        Ok(())
    }
}
