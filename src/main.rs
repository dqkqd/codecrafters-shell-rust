use anyhow::Result;
use codecrafters_shell::run;

fn main() -> Result<()> {
    loop {
        run()?;
    }
}
