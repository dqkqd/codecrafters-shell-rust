mod command;
mod complete;
mod io;
mod parse;
mod shell;
pub(crate) mod utils;

pub use shell::run_shell;

pub const HIST_FILE: &str = "./history";
