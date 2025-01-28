use std::io::{StdoutLock, Write};

use anyhow::Result;

use super::{
    completer, is_whitespace, key::Key, ParsedStatus, BACKSLASH, DOUBLE_QUOTE, NEWLINE,
    SINGLE_QUOTE, SPACE, TAB,
};

pub(super) trait RawTokenParse {
    fn parse(self) -> Result<ParsedStatus>;
}

pub(super) enum RawQuoteParser<'a> {
    None(NoQuote<'a>),
    Single(SingleQuote<'a>),
    Double(DoubleQuote<'a>),
}

impl<'a> RawQuoteParser<'a> {
    pub fn no_quote(
        stdout: &'a mut StdoutLock<'static>,
        ch: Option<char>,
        escape: bool,
    ) -> RawQuoteParser<'a> {
        RawQuoteParser::None(NoQuote {
            stdout,
            token: match ch {
                Some(ch) => String::from(ch),
                None => String::new(),
            },
            escape,
        })
    }

    pub fn single_quote(
        stdout: &'a mut StdoutLock<'static>,
        ch: Option<char>,
    ) -> RawQuoteParser<'a> {
        RawQuoteParser::Single(SingleQuote {
            stdout,
            token: match ch {
                Some(ch) => String::from(ch),
                None => String::new(),
            },
        })
    }

    pub fn double_quote(
        stdout: &'a mut StdoutLock<'static>,
        ch: Option<char>,
    ) -> RawQuoteParser<'a> {
        RawQuoteParser::Double(DoubleQuote {
            stdout,
            token: match ch {
                Some(ch) => String::from(ch),
                None => String::new(),
            },
        })
    }
}

impl RawTokenParse for RawQuoteParser<'_> {
    fn parse(self) -> Result<ParsedStatus> {
        match self {
            RawQuoteParser::None(no_quote) => no_quote.parse(),
            RawQuoteParser::Single(single_quote) => single_quote.parse(),
            RawQuoteParser::Double(double_quote) => double_quote.parse(),
        }
    }
}

pub(super) struct NoQuote<'a> {
    stdout: &'a mut StdoutLock<'static>,
    token: String,
    escape: bool,
}

pub(super) struct SingleQuote<'a> {
    stdout: &'a mut StdoutLock<'static>,
    token: String,
}

pub(super) struct DoubleQuote<'a> {
    stdout: &'a mut StdoutLock<'static>,
    token: String,
}

impl RawTokenParse for NoQuote<'_> {
    fn parse(mut self) -> Result<ParsedStatus> {
        loop {
            if self.escape {
                // reading escape character
                match Key::read(self.stdout)? {
                    Key::Char(ch) => self.token.push(ch),
                    Key::Newline => self.token.push(NEWLINE),
                    Key::Tab => self.token.push(TAB),
                    Key::Backspace => todo!("handle backspace"),
                };
                self.escape = false;
            } else {
                match Key::read(self.stdout)? {
                    Key::Char(BACKSLASH) => {
                        // Handle escape in the next read.
                        self.escape = true;
                    }
                    Key::Char(ch) if is_whitespace(ch) => break,
                    Key::Char(ch) => self.token.push(ch),
                    Key::Tab => {
                        if let Some(mut suffix) =
                            completer::completed_suffix(self.stdout, &self.token)?
                        {
                            self.token += &suffix;
                            suffix.push(SPACE);
                            self.stdout.write_all(suffix.as_bytes())?;
                            self.stdout.flush()?;
                            break;
                        }
                    }
                    Key::Newline => return Ok(ParsedStatus::Stop(self.token)),
                    Key::Backspace => todo!("handle backspace"),
                }
            }
        }

        Ok(ParsedStatus::Continue(self.token))
    }
}

impl RawTokenParse for SingleQuote<'_> {
    fn parse(mut self) -> Result<ParsedStatus> {
        loop {
            match Key::read(self.stdout)? {
                Key::Char(SINGLE_QUOTE) => break,
                Key::Char(ch) => self.token.push(ch),
                Key::Tab => {
                    if let Some(suffix) = completer::completed_suffix(self.stdout, &self.token)? {
                        self.token += &suffix;
                        self.stdout.write_all(suffix.as_bytes())?;
                        self.stdout.flush()?;
                    }
                }
                Key::Newline => self.token.push(NEWLINE),
                Key::Backspace => todo!("handle backspace"),
            }
        }
        Ok(ParsedStatus::Continue(self.token))
    }
}

impl RawTokenParse for DoubleQuote<'_> {
    fn parse(mut self) -> Result<ParsedStatus> {
        loop {
            match Key::read(self.stdout)? {
                Key::Char(DOUBLE_QUOTE) => break,
                Key::Char(BACKSLASH) => match Key::read(self.stdout)? {
                    Key::Char(ch) if "$`\"\\\n".contains(ch) => self.token.push(ch),
                    Key::Char(ch) => self.token.extend([BACKSLASH, ch]),
                    Key::Tab => self.token.push(TAB),
                    Key::Newline => self.token.push(NEWLINE),
                    Key::Backspace => todo!("handle backspace"),
                },
                Key::Char(ch) => self.token.push(ch),
                Key::Tab => {
                    if let Some(suffix) = completer::completed_suffix(self.stdout, &self.token)? {
                        self.token += &suffix;
                        self.stdout.write_all(suffix.as_bytes())?;
                        self.stdout.flush()?;
                    }
                }
                Key::Newline => self.token.push(NEWLINE),
                Key::Backspace => todo!("handle backspace"),
            }
        }
        Ok(ParsedStatus::Continue(self.token))
    }
}
