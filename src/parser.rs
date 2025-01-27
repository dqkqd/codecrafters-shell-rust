mod completer;
mod key;
mod quotes;
mod raw;
mod token;

pub(crate) use raw::parse_raw_tokens;
pub(crate) use token::parse_tokens;

pub(crate) use token::{RedirectToken, ValueToken};

const BACKSLASH: char = '\\';
const NEWLINE: char = '\n';
const SINGLE_QUOTE: char = '\'';
const DOUBLE_QUOTE: char = '"';
const TAB: char = '\t';
const SPACE: char = ' ';

pub(super) enum ParsedStatus {
    Continue(String),
    Stop(String),
}

fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\r' || c == '\n'
}
