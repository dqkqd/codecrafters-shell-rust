use anyhow::Result;
use std::io::{self, Write};

use crate::command::Command;

#[derive(Debug)]
pub struct Shell {
    stdout: io::Stdout,
}

impl Default for Shell {
    fn default() -> Self {
        Self {
            stdout: io::stdout(),
        }
    }
}

impl Shell {
    pub fn new() -> Shell {
        Shell::default()
    }

    pub fn run(&mut self, input: io::Stdin) -> Result<()> {
        loop {
            // first write dollar sign
            self.prompt()?;

            // read user input
            let mut raw_input = String::new();
            match input.read_line(&mut raw_input)? {
                0 => break Ok(()),
                _ => {
                    let mut raw_input = raw_input.trim();
                    if raw_input.is_empty() {
                        continue;
                    }
                    let mut command = Command::parse(&mut raw_input)?;
                    command.execute()?;
                }
            }
        }
    }

    fn prompt(&mut self) -> Result<()> {
        self.write_and_flush("$ ")
    }

    fn write_and_flush(&mut self, data: &str) -> Result<()> {
        self.stdout.write_all(data.as_bytes())?;
        self.stdout.flush()?;
        Ok(())
    }
}
