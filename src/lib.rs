mod command;
mod error;

use anyhow::Result;

pub use command::Command;

pub trait Execute {
    fn execute(self) -> Result<()>;
}
