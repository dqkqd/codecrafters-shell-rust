use winnow::{
    ascii::multispace0,
    combinator::{alt, delimited, opt, preceded, repeat},
    token::{any, take, take_till, take_until},
    ModalResult, Parser,
};

use super::{CommandArg, ParseInput};

pub(super) fn raw_command_arg(input: ParseInput) -> ModalResult<CommandArg> {
    preceded(
        multispace0,
        repeat(1.., alt((single_quote, double_quote, no_quote)))
            .fold(String::new, |acc, item| acc + &item)
            .map(CommandArg),
    )
    .parse_next(input)
}

fn single_quote(input: ParseInput) -> ModalResult<String> {
    delimited('\'', take_until(1.., "'").map(String::from), '\'').parse_next(input)
}

fn double_quote(input: ParseInput) -> ModalResult<String> {
    delimited(
        '"',
        repeat(0.., double_quote_inner)
            .fold(String::new, |acc, item| acc + &item)
            .verify(|s: &str| !s.is_empty()),
        '"',
    )
    .parse_next(input)
}

fn double_quote_inner(input: ParseInput) -> ModalResult<String> {
    let token = take_till(0.., |c: char| "\"\\".contains(c)).map(String::from);
    let backslash = opt(alt((
        (preceded("\\", "$")),
        (preceded("\\", "`")),
        (preceded("\\", "\"")),
        (preceded("\\", "\\")),
        (preceded("\\", "\n")),
        take(2usize).verify(|c: &str| c.starts_with("\\")),
    )));

    (token, backslash)
        .map(|(token, s)| match s {
            Some(s) => token + s,
            None => token,
        })
        .verify(|s: &str| !s.is_empty())
        .parse_next(input)
}

fn no_quote(input: ParseInput) -> ModalResult<String> {
    repeat(0.., no_quote_inner)
        .fold(String::new, |acc, item| acc + &item)
        .verify(|s: &str| !s.is_empty())
        .parse_next(input)
}

fn no_quote_inner(input: ParseInput) -> ModalResult<String> {
    let token = take_till(0.., |c: char| " \t\\\'\"".contains(c)).map(String::from);
    let backslash = opt(preceded("\\", any));

    (token, backslash)
        .map(|(mut token, ch)| {
            if let Some(ch) = ch {
                token.push(ch);
            };
            token
        })
        .verify(|s: &str| !s.is_empty())
        .parse_next(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_raw_command_arg() {
        assert_eq!(
            raw_command_arg(&mut "hello").unwrap(),
            CommandArg("hello".into())
        );
        assert_eq!(
            raw_command_arg(&mut "hello world").unwrap(),
            CommandArg("hello".into())
        );
        assert_eq!(
            raw_command_arg(&mut "'hello world'").unwrap(),
            CommandArg("hello world".into())
        );
        assert_eq!(
            raw_command_arg(&mut "'hello' world").unwrap(),
            CommandArg("hello".into())
        );
        assert_eq!(
            raw_command_arg(&mut "hello'world'").unwrap(),
            CommandArg("helloworld".into())
        );
        assert_eq!(
            raw_command_arg(&mut "\"hello world\"").unwrap(),
            CommandArg("hello world".into())
        );
        assert_eq!(
            raw_command_arg(&mut "\"hello\" world\"").unwrap(),
            CommandArg("hello".into())
        );
        assert_eq!(
            raw_command_arg(&mut "\"hello\\\" world\"").unwrap(),
            CommandArg("hello\" world".into())
        );
        assert_eq!(
            raw_command_arg(&mut "\"hello\\$ world\"").unwrap(),
            CommandArg("hello$ world".into())
        );
        assert_eq!(
            raw_command_arg(&mut "\"hello\\` world\"").unwrap(),
            CommandArg("hello` world".into())
        );
        // // assert_eq!(
        // //     parse_arg(&mut "\"hello\\\n world\"").unwrap(),
        // //     "hello\n world"
        // // );
        assert_eq!(
            raw_command_arg(&mut "\"hello\\x world\"").unwrap(),
            CommandArg("hello\\x world".into())
        );
        assert_eq!(
            raw_command_arg(&mut "\"hello\\$\"").unwrap(),
            CommandArg("hello$".into())
        );

        assert_eq!(
            raw_command_arg(&mut "hello\\ world").unwrap(),
            CommandArg("hello world".into())
        );

        assert_eq!(
            raw_command_arg(&mut "'hello\\\\world'").unwrap(),
            CommandArg("hello\\\\world".into())
        );

        assert!(raw_command_arg(&mut " ").is_err())
    }
}
