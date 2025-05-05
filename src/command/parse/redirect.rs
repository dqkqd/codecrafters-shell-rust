use winnow::{
    ascii::{digit1, multispace0},
    combinator::{alt, opt},
    ModalResult, Parser,
};

use super::{command::raw_command_arg, ParseInput, RedirectArg};

pub(super) fn raw_redirect_arg(input: ParseInput) -> ModalResult<RedirectArg> {
    alt((redirect_output, redirect_output)).parse_next(input)
}

pub(super) fn redirect_output(input: ParseInput) -> ModalResult<RedirectArg> {
    let (n, _, _, _, word) = (
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
    }
}
