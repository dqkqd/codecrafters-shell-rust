use anyhow::Result;
use std::io::{BufRead, Write};

#[derive(Debug, Default)]
pub(crate) struct ShellOption {
    print_input: bool,
}

impl ShellOption {
    #[cfg(test)]
    pub fn with_print_input(mut self, print_input: bool) -> ShellOption {
        self.print_input = print_input;
        self
    }
}

#[derive(Debug)]
pub struct Shell<W: Write> {
    pub(crate) writer: W,
    opts: ShellOption,
}

impl<W: Write> Shell<W> {
    pub fn new(writer: W) -> Shell<W> {
        Shell {
            writer,
            opts: ShellOption::default(),
        }
    }

    pub fn run<R: BufRead>(&mut self, mut input: R) -> Result<()> {
        loop {
            // first write dollar sign
            self.prompt()?;

            // read user input
            let mut raw_input = String::new();
            let n = input.read_line(&mut raw_input)?;
            match n {
                0 => break Ok(()),
                1 => {
                    self.add_input(&raw_input)?;
                }
                2.. => {
                    self.add_input(&raw_input)?;
                    if let Some('\n') = raw_input.chars().last() {
                        raw_input.pop();
                    }
                    self.write_and_flush(&format!("{raw_input}: command not found\n"))?;
                }
            }
        }
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
    pub(crate) fn with_option(mut self, opts: ShellOption) -> Shell<W> {
        self.opts = opts;
        self
    }
}

#[cfg(test)]
fn run_test_with_input(input: &str) -> anyhow::Result<String> {
    use std::io::BufReader;

    use super::*;

    let input = BufReader::new(input.as_bytes());
    let mut shell =
        Shell::new(Vec::new()).with_option(ShellOption::default().with_print_input(true));
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
        let output = run_test_with_input("some_command\n")?;
        assert_eq!(
            output,
            r"$ some_command
some_command: command not found
$ "
        );
        Ok(())
    }

    #[test]
    fn repl() -> anyhow::Result<()> {
        let output = run_test_with_input(
            r#"
command1
command2
"#,
        )?;
        assert_eq!(
            output,
            r"$ 
$ command1
command1: command not found
$ command2
command2: command not found
$ "
        );
        Ok(())
    }

    #[test]
    fn repl_empty() -> anyhow::Result<()> {
        let output = run_test_with_input(
            r#"
command1

"#,
        )?;
        assert_eq!(
            output,
            r#"$ 
$ command1
command1: command not found
$ 
$ "#
        );
        Ok(())
    }
}
