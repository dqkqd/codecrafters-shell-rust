use anyhow::Result;
use codecrafters_shell::run;
use std::io::{self, Write};

fn main() -> Result<()> {
    // Uncomment this block to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        run(&input)?;
    }
}
