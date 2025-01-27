use std::path::PathBuf;

use anyhow::{Context, Result};

use super::{Execute, ExecutedOutput};

pub struct ExecFileCmd {
    pub command: String,
    pub path: PathBuf,
    pub args: Vec<String>,
}

impl ExecFileCmd {
    pub fn new(command: String, args: Vec<String>) -> Result<ExecFileCmd> {
        let path = std::env::var("PATH")?;
        let path = path
            .split(":")
            .map(|path| PathBuf::from(path).join(&command))
            .find(|path| path.is_file())
            .with_context(|| "missing executable file command")?;

        Ok(ExecFileCmd {
            command,
            path,
            args,
        })
    }
}

impl Execute for ExecFileCmd {
    fn execute(self) -> Result<ExecutedOutput> {
        let output = std::process::Command::new(self.command)
            .args(self.args)
            .output()?;
        let (stdout, stderr) = (output.stdout, output.stderr);
        Ok(ExecutedOutput::new()
            .with_stdout(stdout)
            .with_stderr(stderr))
    }
}
