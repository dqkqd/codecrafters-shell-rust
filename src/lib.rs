mod command;
mod error;

use anyhow::Result;

pub use command::Cmd;

pub trait Execute {
    fn execute(self) -> Result<()>;
}
