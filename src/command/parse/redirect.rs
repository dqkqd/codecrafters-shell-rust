use std::fs::OpenOptions;

use anyhow::bail;
use winnow::{
    ascii::{digit1, multispace0},
    combinator::{alt, opt},
    ModalResult, Parser,
};

use crate::command::io::{PErr, POut, PType};

use super::{command::command_arg, ParseInput, RedirectArg};

pub(super) fn redirect_arg(input: ParseInput) -> ModalResult<RedirectArg> {
    alt((append_output, output)).parse_next(input)
}

impl RedirectArg {
    pub(super) fn into_pipe(self) -> anyhow::Result<PType> {
        match self {
            RedirectArg::Output { n, word } => {
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
            RedirectArg::AppendOutput { n, word } => {
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

fn output(input: ParseInput) -> ModalResult<RedirectArg> {
    let (_, n, _, _, _, word) = (
        multispace0,
        opt(digit1).map(|s| s.map(|s: &str| s.parse::<i32>().unwrap()).unwrap_or(1)),
        ">",
        opt("|"),
        multispace0,
        command_arg,
    )
        .parse_next(input)?;
    Ok(RedirectArg::Output { n, word: word.0 })
}

fn append_output(input: ParseInput) -> ModalResult<RedirectArg> {
    let (_, n, _, _, word) = (
        multispace0,
        opt(digit1).map(|s| s.map(|s: &str| s.parse::<i32>().unwrap()).unwrap_or(1)),
        ">>",
        multispace0,
        command_arg,
    )
        .parse_next(input)?;
    Ok(RedirectArg::AppendOutput { n, word: word.0 })
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_output() {
        assert_eq!(
            redirect_arg(&mut ">word").unwrap(),
            RedirectArg::Output {
                n: 1,
                word: "word".into()
            }
        );
        assert_eq!(
            redirect_arg(&mut "1>word").unwrap(),
            RedirectArg::Output {
                n: 1,
                word: "word".into()
            }
        );
        assert_eq!(
            redirect_arg(&mut "2>word").unwrap(),
            RedirectArg::Output {
                n: 2,
                word: "word".into()
            }
        );
        assert_eq!(
            redirect_arg(&mut ">|word").unwrap(),
            RedirectArg::Output {
                n: 1,
                word: "word".into()
            }
        );
        assert_eq!(
            redirect_arg(&mut "> word").unwrap(),
            RedirectArg::Output {
                n: 1,
                word: "word".into()
            }
        );
    }

    #[test]
    fn test_append_output() {
        assert_eq!(
            redirect_arg(&mut ">>word").unwrap(),
            RedirectArg::AppendOutput {
                n: 1,
                word: "word".into()
            }
        );
        assert_eq!(
            redirect_arg(&mut ">> word").unwrap(),
            RedirectArg::AppendOutput {
                n: 1,
                word: "word".into()
            }
        );
        assert_eq!(
            redirect_arg(&mut "1>>word").unwrap(),
            RedirectArg::AppendOutput {
                n: 1,
                word: "word".into()
            }
        );
        assert_eq!(
            redirect_arg(&mut "2>>word").unwrap(),
            RedirectArg::AppendOutput {
                n: 2,
                word: "word".into()
            }
        );
    }
}
