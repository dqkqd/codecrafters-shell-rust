use std::io::{self, BufReader, BufWriter};

use codecrafters_shell::Shell;

fn main() -> anyhow::Result<()> {
    let reader = BufReader::new(io::stdin().lock());
    let writer = BufWriter::new(io::stdout().lock());
    Shell::new(writer).run(reader)?;
    Ok(())
}
