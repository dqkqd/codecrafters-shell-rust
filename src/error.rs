use std::num::ParseIntError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("parse int error")]
    ParseIntError(#[from] ParseIntError),
}
