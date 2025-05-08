use std::fs::OpenOptions;

use anyhow::bail;
use winnow::{
    ascii::{digit1, multispace0},
    combinator::{alt, opt},
    ModalResult, Parser,
};

use crate::io::{PErr, POut, PType};

use super::{command::command_token, RedirectToken, Stream};

pub(super) fn redirect_token(stream: &mut Stream) -> ModalResult<RedirectToken> {
    alt((append_output, output)).parse_next(stream)
}

impl RedirectToken {
    pub(super) fn into_pipe(self) -> anyhow::Result<PType> {
        match self {
            RedirectToken::Output { n, word } => {
                let file = OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .append(false)
                    .write(true)
                    .open(word)?;
                match n {
                    1 => Ok(PType::Out(POut::File(file))),
                    2 => Ok(PType::Err(PErr::File(file))),
                    _ => bail!("invalid file descriptor {n}"),
                }
            }
            RedirectToken::AppendOutput { n, word } => {
                let file = OpenOptions::new().create(true).append(true).open(word)?;
                match n {
                    1 => Ok(PType::Out(POut::File(file))),
                    2 => Ok(PType::Err(PErr::File(file))),
                    _ => bail!("invalid file descriptor {n}"),
                }
            }
        }
    }
}

fn output(stream: &mut Stream) -> ModalResult<RedirectToken> {
    let (_, n, _, _, _, word) = (
        multispace0,
        opt(digit1).map(|s| s.map(|s: &str| s.parse::<i32>().unwrap()).unwrap_or(1)),
        ">",
        opt("|"),
        multispace0,
        command_token,
    )
        .parse_next(stream)?;
    Ok(RedirectToken::Output { n, word: word.0 })
}

fn append_output(stream: &mut Stream) -> ModalResult<RedirectToken> {
    let (_, n, _, _, word) = (
        multispace0,
        opt(digit1).map(|s| s.map(|s: &str| s.parse::<i32>().unwrap()).unwrap_or(1)),
        ">>",
        multispace0,
        command_token,
    )
        .parse_next(stream)?;
    Ok(RedirectToken::AppendOutput { n, word: word.0 })
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_output() {
        assert_eq!(
            redirect_token(&mut Stream::new(">word\n")).unwrap(),
            RedirectToken::Output {
                n: 1,
                word: "word".into()
            }
        );
        assert_eq!(
            redirect_token(&mut Stream::new("1>word\n")).unwrap(),
            RedirectToken::Output {
                n: 1,
                word: "word".into()
            }
        );
        assert_eq!(
            redirect_token(&mut Stream::new("2>word\n")).unwrap(),
            RedirectToken::Output {
                n: 2,
                word: "word".into()
            }
        );
        assert_eq!(
            redirect_token(&mut Stream::new(">|word\n")).unwrap(),
            RedirectToken::Output {
                n: 1,
                word: "word".into()
            }
        );
        assert_eq!(
            redirect_token(&mut Stream::new("> word\n")).unwrap(),
            RedirectToken::Output {
                n: 1,
                word: "word".into()
            }
        );
    }

    #[test]
    fn test_append_output() {
        assert_eq!(
            redirect_token(&mut Stream::new(">>word\n")).unwrap(),
            RedirectToken::AppendOutput {
                n: 1,
                word: "word".into()
            }
        );
        assert_eq!(
            redirect_token(&mut Stream::new(">> word\n")).unwrap(),
            RedirectToken::AppendOutput {
                n: 1,
                word: "word".into()
            }
        );
        assert_eq!(
            redirect_token(&mut Stream::new("1>>word\n")).unwrap(),
            RedirectToken::AppendOutput {
                n: 1,
                word: "word".into()
            }
        );
        assert_eq!(
            redirect_token(&mut Stream::new("2>>word\n")).unwrap(),
            RedirectToken::AppendOutput {
                n: 2,
                word: "word".into()
            }
        );
    }
}
