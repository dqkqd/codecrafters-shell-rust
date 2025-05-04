use std::io::{self, BufReader, BufWriter, Write};

use codecrafters_shell::run_shell;

fn main() -> anyhow::Result<()> {
    // Uncomment this block to pass the first stage

    let input = io::stdin().lock();
    let mut input = BufReader::new(input);
    let output = io::stdout().lock();
    let mut output = BufWriter::new(output);

    run_shell(&mut input, &mut output)?;
    output.flush()?;

    // // Wait for user input
    // let mut input = String::new();
    // io::stdin().read_line(&mut input).unwrap();

    Ok(())
}
