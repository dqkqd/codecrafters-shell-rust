use std::io::{BufRead, Write};

pub fn run_shell<R, W>(input: &mut R, output: &mut W) -> anyhow::Result<()>
where
    R: BufRead,
    W: Write,
{
    // first write dollar sign
    output.write_all(b"$ ")?;
    output.flush()?;

    // read user input
    let mut raw_string = String::new();
    input.read_line(&mut raw_string)?;

    Ok(())
}

#[cfg(test)]
fn run_test_with_input(input: &str) -> anyhow::Result<String> {
    use std::io::BufReader;

    let mut input = BufReader::new(input.as_bytes());
    let mut output = Vec::new();
    run_shell(&mut input, &mut output)?;

    let output = String::from_utf8(output)?;
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
}
