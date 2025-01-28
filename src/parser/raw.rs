use std::io::{self, StdoutLock};

use anyhow::Result;
use crossterm::terminal;

use crate::parser::TAB;

use super::{
    is_whitespace,
    key::Key,
    quotes::{RawQuoteParser, RawTokenParse},
    ParsedStatus, BACKSLASH, DOUBLE_QUOTE, SINGLE_QUOTE, SPACE,
};

pub(crate) fn parse_raw_tokens() -> Result<Vec<String>> {
    let mut parser = RawTokenParser::new()?;
    loop {
        match Key::read(&mut parser.stdout)? {
            Key::Char(SINGLE_QUOTE) => {
                // Single quote never trigger stop.
                let status =
                    RawQuoteParser::single_quote(&mut parser.stdout, None, &parser.raw).parse()?;
                if let ParsedStatus::Continue(s) = status {
                    parser.add(s);
                }
            }
            Key::Char(DOUBLE_QUOTE) => {
                // Double quote never trigger stop.
                let status =
                    RawQuoteParser::double_quote(&mut parser.stdout, None, &parser.raw).parse()?;
                if let ParsedStatus::Continue(s) = status {
                    parser.add(s);
                }
            }
            Key::Char(ch) if is_whitespace(ch) => parser.add(ch.to_string()),
            Key::Char(ch) => {
                let token_parser = if ch == BACKSLASH {
                    RawQuoteParser::no_quote(&mut parser.stdout, None, true, &parser.raw)
                } else {
                    RawQuoteParser::no_quote(&mut parser.stdout, Some(ch), false, &parser.raw)
                };

                let status = token_parser.parse()?;

                match status {
                    ParsedStatus::Continue(s) => {
                        // No quote always ended with space.
                        parser.add(s);
                        parser.add(SPACE.to_string());
                    }
                    ParsedStatus::Stop(s) => {
                        parser.add(s);
                        break;
                    }
                }
            }
            Key::Tab => parser.add(TAB.to_string()),
            Key::Newline => break,
            Key::Backspace => todo!("handle backspace"),
        };
    }

    // concatenate together strings that are not separated by space
    let tokens = parser
        .tokens
        .split(|s| s == &SPACE.to_string())
        .map(|s| s.concat())
        .collect();

    Ok(tokens)
}

pub(crate) struct RawTokenParser {
    stdout: StdoutLock<'static>,
    tokens: Vec<String>,
    raw: String,
}

impl RawTokenParser {
    fn new() -> Result<RawTokenParser> {
        terminal::enable_raw_mode()?;

        let stdout = io::stdout().lock();
        let parser = RawTokenParser {
            stdout,
            tokens: Vec::new(),
            raw: String::new(),
        };

        Ok(parser)
    }

    fn add(&mut self, s: String) {
        self.raw += &s;

        // do not add two consecutive white space to tokens
        if s.trim().is_empty() {
            if self
                .tokens
                .last()
                .is_some_and(|last| last != &SPACE.to_string())
            {
                self.tokens.push(SPACE.to_string());
            }
        } else {
            self.tokens.push(s);
        }
    }
}

impl Drop for RawTokenParser {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("cannot disable raw mode");
    }
}
