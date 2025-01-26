use std::{env::VarError, num::ParseIntError, string::FromUtf8Error};

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

    #[error("from utf 8 error {0}")]
    FromUtf8Error(#[from] FromUtf8Error),
}
