use std::io::{self};

use codecrafters_shell::Shell;

fn main() -> anyhow::Result<()> {
    Shell::new().run(io::stdin())?;
    Ok(())
}
