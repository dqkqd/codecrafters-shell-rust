mod raw;

use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use raw::RawToken;
use raw::RawTokenParser;
use raw::WHITESPACE;

use crate::error::CmdError;

pub(crate) fn parse_tokens(s: &str) -> Result<(Vec<RedirectToken>, Vec<ValueToken>)> {
    let raw_token_parser = RawTokenParser::new(s);
    let raw_tokens = raw_token_parser.parse();

    let parser = TokenParser::new(raw_tokens);
    let tokens = parser.parse()?;

    let mut redirects = Vec::new();
    let mut values = Vec::new();
    for token in tokens {
        match token {
            Token::Redirect(redirect_token) => redirects.push(redirect_token),
            Token::Value(value_token) => values.push(value_token),
        }
    }

    Ok((redirects, values))
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Redirect(RedirectToken),
    Value(ValueToken),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RedirectToken {
    Stdout(PathBuf),
    Stderr(PathBuf),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ValueToken(pub Vec<u8>);

pub struct TokenParser {
    input: Vec<RawToken>,
    pos: usize,
}

impl RedirectToken {
    pub fn path(&self) -> &Path {
        match self {
            RedirectToken::Stdout(path_buf) => path_buf.as_path(),
            RedirectToken::Stderr(path_buf) => path_buf.as_path(),
        }
    }
}

impl ValueToken {
    pub fn concat(values: Vec<ValueToken>) -> Result<String> {
        let values: Vec<u8> = values.into_iter().fold(Vec::new(), |mut acc, e| {
            if !acc.is_empty() {
                acc.push(WHITESPACE);
            }
            acc.extend(e.0);
            acc
        });

        let value = String::from_utf8(values)?;
        Ok(value)
    }
}

impl TryFrom<ValueToken> for String {
    type Error = CmdError;

    fn try_from(value: ValueToken) -> std::result::Result<Self, Self::Error> {
        let s = String::from_utf8(value.0)?;
        Ok(s)
    }
}

impl TokenParser {
    fn new(tokens: Vec<RawToken>) -> TokenParser {
        TokenParser {
            input: tokens,
            pos: 0,
        }
    }

    fn next(&mut self) -> Option<RawToken> {
        let c = self.input.get(self.pos).cloned()?;
        self.pos += 1;
        Some(c)
    }

    fn parse(mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        let token_to_path = |t: RawToken| -> Result<PathBuf> {
            let path = String::from_utf8(t.into_inner())?;
            Ok(PathBuf::from(path))
        };

        loop {
            let token = match self.next() {
                Some(RawToken(t)) if t == b">" || t == b"1>" => {
                    let token = self.next().with_context(|| "invalid redirect token")?;
                    let redirect_token = RedirectToken::Stdout(token_to_path(token)?);
                    Token::Redirect(redirect_token)
                }
                Some(RawToken(t)) if t == b"2>" => {
                    let token = self.next().with_context(|| "invalid redirect token")?;
                    let redirect_token = RedirectToken::Stderr(token_to_path(token)?);
                    Token::Redirect(redirect_token)
                }
                Some(t) => Token::Value(ValueToken(t.into_inner())),
                None => break,
            };

            tokens.push(token);
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod test {
    use raw::RawTokenParser;

    use super::*;

    macro_rules! T {
        (stdout, $s:literal) => {
            Token::Redirect(RedirectToken::Stdout($s.into()))
        };
        (stderr, $s:literal) => {
            Token::Redirect(RedirectToken::Stderr($s.into()))
        };
        (value, $s:literal) => {
            Token::Value(ValueToken($s.into()))
        };
    }

    #[test]
    fn test_parse_redirect_stdout_default() -> Result<()> {
        let parser = RawTokenParser::new(r#"ls /tmp/baz > /tmp/foo/baz.md"#);
        let tokens = parser.parse();
        let parser = TokenParser::new(tokens);
        let tokens = parser.parse()?;
        assert_eq!(
            tokens,
            [
                T!(value, "ls"),
                T!(value, "/tmp/baz"),
                T!(stdout, "/tmp/foo/baz.md")
            ]
        );
        Ok(())
    }

    #[test]
    fn test_parse_redirect_stdout() -> Result<()> {
        let parser = RawTokenParser::new(r#"ls /tmp/baz > /tmp/foo/baz.md"#);
        let tokens = parser.parse();
        let parser = TokenParser::new(tokens);
        let tokens = parser.parse()?;
        assert_eq!(
            tokens,
            [
                T!(value, "ls"),
                T!(value, "/tmp/baz"),
                T!(stdout, "/tmp/foo/baz.md")
            ]
        );
        Ok(())
    }

    #[test]
    fn test_parse_redirect_stderr() -> Result<()> {
        let parser = RawTokenParser::new(r#"ls /tmp/baz 2> /tmp/foo/baz.md"#);
        let tokens = parser.parse();
        let parser = TokenParser::new(tokens);
        let tokens = parser.parse()?;
        assert_eq!(
            tokens,
            [
                T!(value, "ls"),
                T!(value, "/tmp/baz"),
                T!(stderr, "/tmp/foo/baz.md")
            ]
        );
        Ok(())
    }

    #[test]
    fn test_parse_redirect_multiple_stdout_stderr() -> Result<()> {
        let parser = RawTokenParser::new(
            r#"ls /tmp/baz > stdout1 > stdout2 1> stdout3 2> stderr1 2> stderr2"#,
        );
        let tokens = parser.parse();
        let parser = TokenParser::new(tokens);
        let tokens = parser.parse()?;
        assert_eq!(
            tokens,
            [
                T!(value, "ls"),
                T!(value, "/tmp/baz"),
                T!(stdout, "stdout1"),
                T!(stdout, "stdout2"),
                T!(stdout, "stdout3"),
                T!(stderr, "stderr1"),
                T!(stderr, "stderr2"),
            ]
        );
        Ok(())
    }
}
