use anyhow::Result;
use std::io::{BufRead, Write};

#[derive(Debug, Default)]
struct OutputOption {
    print_input: bool,
}

impl OutputOption {
    #[cfg(test)]
    fn with_print_input(mut self, print_input: bool) -> OutputOption {
        self.print_input = print_input;
        self
    }
}

#[derive(Debug)]
pub struct Shell<W: Write> {
    writer: W,
    opts: OutputOption,
}

impl<W: Write> Shell<W> {
    pub fn new(writer: W) -> Shell<W> {
        Shell {
            writer,
            opts: OutputOption::default(),
        }
    }

    pub fn run<R: BufRead>(&mut self, mut input: R) -> Result<()> {
        // first write dollar sign
        self.prompt()?;

        // read user input
        let mut raw_input = String::new();
        let out = input.read_line(&mut raw_input)?;
        if out > 1 {
            self.add_input(&raw_input)?;
            if let Some('\n') = raw_input.chars().last() {
                raw_input.pop();
            }
            self.write_and_flush(&format!("{raw_input}: command not found\n"))?;
        }

        Ok(())
    }

    fn prompt(&mut self) -> Result<()> {
        self.write_and_flush("$ ")
    }

    fn add_input(&mut self, data: &str) -> Result<()> {
        if self.opts.print_input {
            self.write_and_flush(data)?;
        }
        Ok(())
    }

    fn write_and_flush(&mut self, data: &str) -> Result<()> {
        self.writer.write_all(data.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }

    #[cfg(test)]
    fn with_option(mut self, opts: OutputOption) -> Shell<W> {
        self.opts = opts;
        self
    }
}

#[cfg(test)]
fn run_test_with_input(input: &str) -> anyhow::Result<String> {
    use std::io::BufReader;

    let input = input.to_string() + "\n";
    let input = BufReader::new(input.as_bytes());
    let mut shell =
        Shell::new(Vec::new()).with_option(OutputOption::default().with_print_input(true));
    shell.run(input)?;

    let output = String::from_utf8(shell.writer)?;
    Ok(output)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn print_a_prompt() -> anyhow::Result<()> {
        let output = run_test_with_input("")?;
        assert_eq!(output, "$ ");
        Ok(())
    }

    #[test]
    fn handle_invalid_commands() -> anyhow::Result<()> {
        let output = run_test_with_input("some_command")?;
        assert_eq!(
            output,
            r#"$ some_command
some_command: command not found
"#
        );
        Ok(())
    }
}
