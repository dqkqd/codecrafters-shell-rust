use cmd::InternalCommand;
use io::{PErr, PIn, POut};
use parse_cmd::{parse_command, ParseInput};

mod cmd;
mod io;
mod parse_cmd;

trait Execute {
    fn execute(
        &mut self,
        stdin: &mut PIn,
        stdout: &mut POut,
        stderr: &mut PErr,
    ) -> anyhow::Result<()>;
}

pub(crate) struct Command {
    stdin: PIn,
    stdout: POut,
    stderr: PErr,
    inner: InternalCommand,
}

impl Command {
    fn new(stdin: PIn, stdout: POut, stderr: PErr, command: InternalCommand) -> Command {
        Command {
            stdin,
            stdout,
            stderr,
            inner: command,
        }
    }

    pub fn parse(input: ParseInput) -> Command {
        parse_command(input)
    }

    pub fn execute(&mut self) -> anyhow::Result<()> {
        self.inner
            .execute(&mut self.stdin, &mut self.stdout, &mut self.stderr)?;
        Ok(())
    }
}
