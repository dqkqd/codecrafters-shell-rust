use anyhow::Result;

use crate::{command::Cmd, redirect::Redirector, token::parse_tokens};

#[derive(Default)]
pub struct ExecutedOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
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

pub trait Execute {
    fn execute(self) -> Result<ExecutedOutput>;
}

pub fn run(cmd: &str) -> Result<()> {
    let (redirects, values) = parse_tokens(cmd)?;

    let command = Cmd::from_value_tokens(values)?;
    let output = command.execute()?;

    let redirector = Redirector::new(redirects);
    let _ = redirector.write_stdout(output.stdout);
    let _ = redirector.write_stderr(output.stderr);

    Ok(())
}
