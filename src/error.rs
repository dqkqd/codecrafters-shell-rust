use std::{env::VarError, num::ParseIntError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CmdError {
    #[error("parse int error")]
    ParseIntError(#[from] ParseIntError),

    #[error("invalid env var")]
    VarError(#[from] VarError),

    #[error("missing command {0}")]
    MissingCmd(String),

    #[error("empty command")]
    Empty,
}
