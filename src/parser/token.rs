use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;

pub(crate) fn parse_tokens(tokens: Vec<String>) -> Result<(Vec<RedirectToken>, Vec<ValueToken>)> {
    let parser = TokenParser::new(tokens);
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
pub struct ValueToken(pub String);

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RedirectToken {
    Stdout(PathBuf),
    Stderr(PathBuf),
    StdoutAppend(PathBuf),
    StderrAppend(PathBuf),
}

impl RedirectToken {
    pub fn path(&self) -> &Path {
        match self {
            RedirectToken::Stdout(path_buf) => path_buf.as_path(),
            RedirectToken::Stderr(path_buf) => path_buf.as_path(),
            RedirectToken::StdoutAppend(path_buf) => path_buf.as_path(),
            RedirectToken::StderrAppend(path_buf) => path_buf.as_path(),
        }
    }
}

pub struct TokenParser {
    input: Vec<String>,
    pos: usize,
}

impl TokenParser {
    fn new(tokens: Vec<String>) -> TokenParser {
        TokenParser {
            input: tokens,
            pos: 0,
        }
    }

    fn next(&mut self) -> Option<String> {
        let c = self.input.get(self.pos).cloned()?;
        self.pos += 1;
        Some(c)
    }

    fn parse(mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            let token = match self.next() {
                Some(t) if t == ">" || t == "1>" => {
                    let token = self.next().with_context(|| "invalid redirect token")?;
                    let redirect_token = RedirectToken::Stdout(token.into());
                    Token::Redirect(redirect_token)
                }
                Some(t) if t == ">>" || t == "1>>" => {
                    let token = self.next().with_context(|| "invalid redirect token")?;
                    let redirect_token = RedirectToken::StdoutAppend(token.into());
                    Token::Redirect(redirect_token)
                }
                Some(t) if t == "2>" => {
                    let token = self.next().with_context(|| "invalid redirect token")?;
                    let redirect_token = RedirectToken::Stderr(token.into());
                    Token::Redirect(redirect_token)
                }
                Some(t) if t == "2>>" => {
                    let token = self.next().with_context(|| "invalid redirect token")?;
                    let redirect_token = RedirectToken::StderrAppend(token.into());
                    Token::Redirect(redirect_token)
                }
                Some(t) => Token::Value(ValueToken(t)),
                None => break,
            };

            tokens.push(token);
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod test {
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

    fn str_to_vec_string(s: &str) -> Vec<String> {
        s.split_whitespace().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_parse_redirect_stdout_default() -> Result<()> {
        let parser = TokenParser::new(str_to_vec_string(r#"ls /tmp/baz > /tmp/foo/baz.md"#));
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
        let parser = TokenParser::new(str_to_vec_string(r#"ls /tmp/baz > /tmp/foo/baz.md"#));
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
        let parser = TokenParser::new(str_to_vec_string(r#"ls /tmp/baz 2> /tmp/foo/baz.md"#));
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
        let parser = TokenParser::new(str_to_vec_string(
            r#"ls /tmp/baz > stdout1 > stdout2 1> stdout3 2> stderr1 2> stderr2"#,
        ));
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
