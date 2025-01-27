use std::io::{self, StdoutLock};

use anyhow::Result;
use crossterm::terminal;

use super::{
    is_whitespace,
    key::Key,
    quotes::{RawQuoteParser, RawTokenParse},
    ParsedStatus, BACKSLASH, DOUBLE_QUOTE, SINGLE_QUOTE, SPACE,
};

pub(crate) fn parse_raw_tokens() -> Result<Vec<String>> {
    let mut tokens: Vec<String> = Vec::new();

    let mut parser = RawTokenParser::new()?;
    loop {
        match Key::read(&mut parser.stdout)? {
            Key::Char(SINGLE_QUOTE) => {
                // Single quote never trigger stop.
                let status = RawQuoteParser::single_quote(&mut parser.stdout, None).parse()?;
                if let ParsedStatus::Continue(s) = status {
                    tokens.push(s);
                }
            }
            Key::Char(DOUBLE_QUOTE) => {
                // Double quote never trigger stop.
                let status = RawQuoteParser::double_quote(&mut parser.stdout, None).parse()?;
                if let ParsedStatus::Continue(s) = status {
                    tokens.push(s);
                }
            }
            Key::Char(ch) if is_whitespace(ch) => {
                if tokens.last().is_none_or(|s| s != &SPACE.to_string()) {
                    tokens.push(SPACE.to_string());
                }
            }
            Key::Char(ch) => {
                let token_parser = if ch == BACKSLASH {
                    RawQuoteParser::no_quote(&mut parser.stdout, None, true)
                } else {
                    RawQuoteParser::no_quote(&mut parser.stdout, Some(ch), false)
                };

                let status = token_parser.parse()?;

                match status {
                    ParsedStatus::Continue(s) => {
                        // No quote always ended with space.
                        tokens.push(s);
                        tokens.push(SPACE.to_string());
                    }
                    ParsedStatus::Stop(s) => {
                        tokens.push(s);
                        break;
                    }
                }
            }
            Key::Tab => {
                if tokens.last().is_none_or(|s| s != &SPACE.to_string()) {
                    tokens.push(SPACE.to_string());
                }
            }
            Key::Newline => break,
            Key::Backspace => todo!("handle backspace"),
        };
    }

    // concatenate together strings that are not separated by space
    let tokens = tokens
        .split(|s| s == &SPACE.to_string())
        .map(|s| s.concat())
        .collect();

    Ok(tokens)
}

pub(crate) struct RawTokenParser {
    // simulate
    stdout: StdoutLock<'static>,
}

impl RawTokenParser {
    fn new() -> Result<RawTokenParser> {
        terminal::enable_raw_mode()?;

        let stdout = io::stdout().lock();
        let parser = RawTokenParser { stdout };

        Ok(parser)
    }
}

impl Drop for RawTokenParser {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("cannot disable raw mode");
    }
}
