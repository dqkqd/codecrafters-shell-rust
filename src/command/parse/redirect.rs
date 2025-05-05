use std::fs::OpenOptions;

use anyhow::bail;
use winnow::{
    ascii::{digit1, multispace0},
    combinator::{alt, opt},
    ModalResult, Parser,
};

use crate::command::io::{PErr, POut, PType};

use super::{command::raw_command_arg, ParseInput, RedirectArg};

pub(super) fn raw_redirect_arg(input: ParseInput) -> ModalResult<RedirectArg> {
    alt((redirect_output, redirect_output)).parse_next(input)
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
            RedirectArg::AppendOutput { n, word } => todo!(),
            RedirectArg::OutputAndError { word } => todo!(),
            RedirectArg::AppendOutputAndError { word } => todo!(),
        }
    }
}

pub(super) fn redirect_output(input: ParseInput) -> ModalResult<RedirectArg> {
    let (_, n, _, _, _, word) = (
        multispace0,
        opt(digit1).map(|s| s.map(|s: &str| s.parse::<i32>().unwrap()).unwrap_or(1)),
        ">",
        opt("|"),
        multispace0,
        raw_command_arg,
    )
        .parse_next(input)?;
    Ok(RedirectArg::Output { n, word: word.0 })
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_raw_redirect_arg() {
        assert_eq!(
            raw_redirect_arg(&mut ">word").unwrap(),
            RedirectArg::Output {
                n: 1,
                word: "word".into()
            }
        );
        assert_eq!(
            raw_redirect_arg(&mut "1>word").unwrap(),
            RedirectArg::Output {
                n: 1,
                word: "word".into()
            }
        );
        assert_eq!(
            raw_redirect_arg(&mut "2>word").unwrap(),
            RedirectArg::Output {
                n: 2,
                word: "word".into()
            }
        );
        assert_eq!(
            raw_redirect_arg(&mut ">|word").unwrap(),
            RedirectArg::Output {
                n: 1,
                word: "word".into()
            }
        );
        assert_eq!(
            raw_redirect_arg(&mut "> word").unwrap(),
            RedirectArg::Output {
                n: 1,
                word: "word".into()
            }
        );
        assert_eq!(
            raw_redirect_arg(&mut "> /this/is/file").unwrap(),
            RedirectArg::Output {
                n: 1,
                word: "/this/is/file".into()
            }
        );
    }
}
