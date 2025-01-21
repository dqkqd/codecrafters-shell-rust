use anyhow::Result;
use codecrafters_shell::{Cmd, Execute};
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

        let command: Cmd = input.try_into().unwrap();
        command.execute()?;
    }
}
