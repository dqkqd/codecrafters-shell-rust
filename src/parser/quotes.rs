use std::io::{StdoutLock, Write};

use anyhow::Result;

use super::{
    completer::{self, CompletedSuffix, TabCompletionState},
    is_whitespace,
    key::Key,
    ParsedStatus, BACKSLASH, DOUBLE_QUOTE, NEWLINE, SINGLE_QUOTE, SPACE, TAB,
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
        raw: &'a str,
    ) -> RawQuoteParser<'a> {
        RawQuoteParser::None(NoQuote {
            raw,
            stdout,
            token: match ch {
                Some(ch) => String::from(ch),
                None => String::new(),
            },
            escape,
            tab_completion_state: TabCompletionState::NotPressed,
        })
    }

    pub fn single_quote(
        stdout: &'a mut StdoutLock<'static>,
        ch: Option<char>,
        raw: &'a str,
    ) -> RawQuoteParser<'a> {
        RawQuoteParser::Single(SingleQuote {
            raw,
            stdout,
            token: match ch {
                Some(ch) => String::from(ch),
                None => String::new(),
            },
            tab_completion_state: TabCompletionState::NotPressed,
        })
    }

    pub fn double_quote(
        stdout: &'a mut StdoutLock<'static>,
        ch: Option<char>,
        raw: &'a str,
    ) -> RawQuoteParser<'a> {
        RawQuoteParser::Double(DoubleQuote {
            raw,
            stdout,
            token: match ch {
                Some(ch) => String::from(ch),
                None => String::new(),
            },
            tab_completion_state: TabCompletionState::NotPressed,
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
    raw: &'a str,
    stdout: &'a mut StdoutLock<'static>,
    token: String,
    escape: bool,
    tab_completion_state: TabCompletionState,
}

pub(super) struct SingleQuote<'a> {
    raw: &'a str,
    stdout: &'a mut StdoutLock<'static>,
    token: String,
    tab_completion_state: TabCompletionState,
}

pub(super) struct DoubleQuote<'a> {
    raw: &'a str,
    stdout: &'a mut StdoutLock<'static>,
    token: String,
    tab_completion_state: TabCompletionState,
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
                let key = Key::read(self.stdout)?;

                match key {
                    Key::Char(BACKSLASH) => {
                        // Handle escape in the next read.
                        self.escape = true;
                    }
                    Key::Char(ch) if is_whitespace(ch) => break,
                    Key::Char(ch) => self.token.push(ch),
                    Key::Tab => {
                        let completed_suffix = completer::completed_suffix(
                            self.stdout,
                            &self.token,
                            self.tab_completion_state,
                            self.raw,
                        )?;

                        if let Some(suffix) = completed_suffix.suffix() {
                            self.token += suffix;
                            self.tab_completion_state = TabCompletionState::NotPressed;
                            self.stdout.write_all(suffix.as_bytes())?;
                            self.stdout.flush()?;
                        } else {
                            // no completed candidate, record state so that subsequent press should
                            // show all matches candidate
                            self.tab_completion_state = TabCompletionState::Pressed
                        }

                        if matches!(completed_suffix, CompletedSuffix::Completed { suffix: _ }) {
                            self.stdout.write_all(SPACE.to_string().as_bytes())?;
                            self.stdout.flush()?;
                            break;
                        }
                    }
                    Key::Newline => return Ok(ParsedStatus::Stop(self.token)),
                    Key::Backspace => todo!("handle backspace"),
                }

                // reset tab completion state
                if !matches!(key, Key::Tab) {
                    self.tab_completion_state = TabCompletionState::NotPressed
                }
            }
        }

        Ok(ParsedStatus::Continue(self.token))
    }
}

impl RawTokenParse for SingleQuote<'_> {
    fn parse(mut self) -> Result<ParsedStatus> {
        loop {
            let key = Key::read(self.stdout)?;

            match key {
                Key::Char(SINGLE_QUOTE) => break,
                Key::Char(ch) => self.token.push(ch),
                Key::Tab => {
                    let completed_suffix = completer::completed_suffix(
                        self.stdout,
                        &self.token,
                        self.tab_completion_state,
                        self.raw,
                    )?;

                    if let Some(suffix) = completed_suffix.suffix() {
                        self.token += suffix;
                        self.tab_completion_state = TabCompletionState::NotPressed;
                        self.stdout.write_all(suffix.as_bytes())?;
                        self.stdout.flush()?;
                    } else {
                        // no completed candidate, record state so that subsequent press should
                        // show all matches candidate
                        self.tab_completion_state = TabCompletionState::Pressed
                    }
                }
                Key::Newline => self.token.push(NEWLINE),
                Key::Backspace => todo!("handle backspace"),
            };
            // reset tab completion state

            if !matches!(key, Key::Tab) {
                self.tab_completion_state = TabCompletionState::NotPressed
            }
        }
        Ok(ParsedStatus::Continue(self.token))
    }
}

impl RawTokenParse for DoubleQuote<'_> {
    fn parse(mut self) -> Result<ParsedStatus> {
        loop {
            let key = Key::read(self.stdout)?;

            match key {
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
                    let completed_suffix = completer::completed_suffix(
                        self.stdout,
                        &self.token,
                        self.tab_completion_state,
                        self.raw,
                    )?;

                    if let Some(suffix) = completed_suffix.suffix() {
                        self.token += suffix;
                        self.tab_completion_state = TabCompletionState::NotPressed;
                        self.stdout.write_all(suffix.as_bytes())?;
                        self.stdout.flush()?;
                    } else {
                        // no completed candidate, record state so that subsequent press should
                        // show all matches candidate
                        self.tab_completion_state = TabCompletionState::Pressed
                    }
                }
                Key::Newline => self.token.push(NEWLINE),
                Key::Backspace => todo!("handle backspace"),
            }

            // reset tab completion state
            if !matches!(key, Key::Tab) {
                self.tab_completion_state = TabCompletionState::NotPressed
            }
        }
        Ok(ParsedStatus::Continue(self.token))
    }
}
