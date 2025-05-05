use std::io;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use winnow::{
    ascii::{digit1, multispace0},
    combinator::{alt, delimited, opt, preceded, repeat},
    token::{any, take, take_till, take_until},
    ModalResult, Parser,
};

use super::io::{PErr, PIn, POut};
use super::{Args, BuiltinCommand, Command, InternalCommand, InvalidCommand, PathCommand};

#[derive(Debug, PartialEq)]
enum RawArg {
    Redirect(RawRedirectArg),
    Command(RawCommandArg),
}

#[derive(Debug, PartialEq)]
struct RawCommandArg(String);

#[derive(Debug, PartialEq)]
enum RawRedirectArg {
    Output { n: i32, word: String },
    AppendOutput { n: i32, word: String },
    OutputAndError { word: String },
    AppendOutputAndError { word: String },
}

pub(super) type ParseInput<'a, 'b> = &'a mut &'b str;

pub(super) fn parse_command(input: ParseInput) -> Command {
    let (cmd, args) = args(input).unwrap_or_else(|_| panic!("cannot parse command {input}"));
    let command = match BuiltinCommand::from_str(&cmd) {
        Ok(builtin) => InternalCommand::Builtin(builtin.with_args(args)),
        Err(_) => match path_lookup(&cmd) {
            Ok(path) => InternalCommand::Path(PathCommand { path, args }),
            Err(_) => InternalCommand::Invalid(InvalidCommand(cmd.to_string())),
        },
    };

    Command::new(
        PIn::Std(io::stdin()),
        POut::Std(io::stdout()),
        PErr::Std(io::stderr()),
        command,
    )
}

pub(super) fn path_lookup(name: &str) -> anyhow::Result<PathBuf> {
    let paths = std::env::var("PATH")?;
    let path = paths
        .split(":")
        .map(|path| PathBuf::from(path).join(name))
        .find(|path| path.is_file())
        .with_context(|| format!("missing executable file command, {name}"))?;
    Ok(path)
}

fn args(input: ParseInput) -> ModalResult<(String, Args)> {
    let args = raw_args.parse_next(input)?;
    let args: Vec<String> = args
        .into_iter()
        .filter_map(|arg| match arg {
            RawArg::Redirect(_) => None,
            RawArg::Command(command) => Some(command.0),
        })
        .collect();
    let (command, args) = args.split_first().unwrap();
    Ok((command.to_string(), Args(args.to_vec())))
}

fn raw_args(input: ParseInput) -> ModalResult<Vec<RawArg>> {
    repeat(
        1..,
        alt((
            raw_redirect_arg.map(RawArg::Redirect),
            raw_command_arg.map(RawArg::Command),
        )),
    )
    .parse_next(input)
}

fn raw_command_arg(input: ParseInput) -> ModalResult<RawCommandArg> {
    preceded(
        multispace0,
        repeat(1.., alt((single_quote, double_quote, no_quote)))
            .fold(String::new, |acc, item| acc + &item)
            .map(RawCommandArg),
    )
    .parse_next(input)
}

fn raw_redirect_arg(input: ParseInput) -> ModalResult<RawRedirectArg> {
    alt((redirect_output, redirect_output)).parse_next(input)
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

fn redirect_output(input: ParseInput) -> ModalResult<RawRedirectArg> {
    let (n, _, _, _, word) = (
        opt(digit1).map(|s| s.map(|s: &str| s.parse::<i32>().unwrap()).unwrap_or(1)),
        ">",
        opt("|"),
        multispace0,
        raw_command_arg,
    )
        .parse_next(input)?;
    Ok(RawRedirectArg::Output { n, word: word.0 })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_args() {
        assert_eq!(
            args(&mut "echo 1 2 3"),
            Ok((
                "echo".into(),
                Args(vec!["1".into(), "2".into(), "3".into()])
            ))
        );
        assert_eq!(
            args(&mut "echo 1 2 3  "),
            Ok((
                "echo".into(),
                Args(vec!["1".into(), "2".into(), "3".into()])
            ))
        );
        assert_eq!(args(&mut "echo"), Ok(("echo".into(), Args(vec![]))));
    }

    #[test]
    fn test_raw_command_arg() {
        assert_eq!(
            raw_command_arg(&mut "hello").unwrap(),
            RawCommandArg("hello".into())
        );
        assert_eq!(
            raw_command_arg(&mut "hello world").unwrap(),
            RawCommandArg("hello".into())
        );
        assert_eq!(
            raw_command_arg(&mut "'hello world'").unwrap(),
            RawCommandArg("hello world".into())
        );
        assert_eq!(
            raw_command_arg(&mut "'hello' world").unwrap(),
            RawCommandArg("hello".into())
        );
        assert_eq!(
            raw_command_arg(&mut "hello'world'").unwrap(),
            RawCommandArg("helloworld".into())
        );
        assert_eq!(
            raw_command_arg(&mut "\"hello world\"").unwrap(),
            RawCommandArg("hello world".into())
        );
        assert_eq!(
            raw_command_arg(&mut "\"hello\" world\"").unwrap(),
            RawCommandArg("hello".into())
        );
        assert_eq!(
            raw_command_arg(&mut "\"hello\\\" world\"").unwrap(),
            RawCommandArg("hello\" world".into())
        );
        assert_eq!(
            raw_command_arg(&mut "\"hello\\$ world\"").unwrap(),
            RawCommandArg("hello$ world".into())
        );
        assert_eq!(
            raw_command_arg(&mut "\"hello\\` world\"").unwrap(),
            RawCommandArg("hello` world".into())
        );
        // // assert_eq!(
        // //     parse_arg(&mut "\"hello\\\n world\"").unwrap(),
        // //     "hello\n world"
        // // );
        assert_eq!(
            raw_command_arg(&mut "\"hello\\x world\"").unwrap(),
            RawCommandArg("hello\\x world".into())
        );
        assert_eq!(
            raw_command_arg(&mut "\"hello\\$\"").unwrap(),
            RawCommandArg("hello$".into())
        );

        assert_eq!(
            raw_command_arg(&mut "hello\\ world").unwrap(),
            RawCommandArg("hello world".into())
        );

        assert_eq!(
            raw_command_arg(&mut "'hello\\\\world'").unwrap(),
            RawCommandArg("hello\\\\world".into())
        );

        assert!(raw_command_arg(&mut " ").is_err())
    }

    #[test]
    fn test_raw_redirect_arg() {
        assert_eq!(
            raw_redirect_arg(&mut ">word").unwrap(),
            RawRedirectArg::Output {
                n: 1,
                word: "word".into()
            }
        );
        assert_eq!(
            raw_redirect_arg(&mut "1>word").unwrap(),
            RawRedirectArg::Output {
                n: 1,
                word: "word".into()
            }
        );
        assert_eq!(
            raw_redirect_arg(&mut "2>word").unwrap(),
            RawRedirectArg::Output {
                n: 2,
                word: "word".into()
            }
        );
        assert_eq!(
            raw_redirect_arg(&mut ">|word").unwrap(),
            RawRedirectArg::Output {
                n: 1,
                word: "word".into()
            }
        );
        assert_eq!(
            raw_redirect_arg(&mut "> word").unwrap(),
            RawRedirectArg::Output {
                n: 1,
                word: "word".into()
            }
        );
    }

    #[test]
    fn test_raw_command_args() {
        assert_eq!(
            raw_args(&mut "hello").unwrap(),
            [RawArg::Command(RawCommandArg("hello".into()))],
        );
        assert_eq!(
            raw_args(&mut "hello world").unwrap(),
            [
                RawArg::Command(RawCommandArg("hello".into())),
                RawArg::Command(RawCommandArg("world".into())),
            ],
        );
        assert_eq!(
            raw_args(&mut "'hello' world").unwrap(),
            [
                RawArg::Command(RawCommandArg("hello".into())),
                RawArg::Command(RawCommandArg("world".into())),
            ],
        );

        assert_eq!(
            raw_args(&mut "'hello world' hello world").unwrap(),
            [
                RawArg::Command(RawCommandArg("hello world".into())),
                RawArg::Command(RawCommandArg("hello".into())),
                RawArg::Command(RawCommandArg("world".into())),
            ],
        );
    }
}
