mod command;
mod parser;
mod redirect;

use std::io::{self, Write};

use anyhow::Result;

use command::{Cmd, Execute};
use parser::{parse_raw_tokens, parse_tokens};
use redirect::Redirector;

pub fn run() -> Result<()> {
    {
        let mut stdout = io::stdout().lock();
        stdout.write_all(b"$ ")?;
        stdout.flush()?;
    }

    let raw_tokens = parse_raw_tokens()?;
    let (redirects, values) = parse_tokens(raw_tokens)?;

    let command = Cmd::from_value_tokens(values)?;
    let output = command.execute()?;

    let redirector = Redirector::new(redirects);
    let _ = redirector.write_stdout(output.stdout);
    let _ = redirector.write_stderr(output.stderr);

    Ok(())
}
