mod command;
mod error;
mod parser;

use anyhow::Result;

pub use command::Cmd;

pub trait Execute {
    fn execute(self) -> Result<()>;
}
