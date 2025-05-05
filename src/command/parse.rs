use std::io;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use winnow::{
    ascii::{multispace0, space0},
    combinator::{delimited, fail, repeat},
    stream::AsChar,
    token::{take, take_till, take_until},
    ModalResult, Parser,
};

use super::io::{PErr, PIn, POut};
use super::{Args, BuiltinCommand, Command, InternalCommand, InvalidCommand, PathCommand};

pub(super) type ParseInput<'a, 'b> = &'a mut &'b str;

pub(super) fn parse_command(input: ParseInput) -> Command {
    let (cmd, args) =
        command_and_args(input).unwrap_or_else(|_| panic!("cannot parse command {input}"));
    let command = match BuiltinCommand::from_str(cmd) {
        Ok(builtin) => InternalCommand::Builtin(builtin.with_args(args)),
        Err(_) => match command_in_path(cmd) {
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

pub(super) fn command_in_path(name: &str) -> anyhow::Result<PathBuf> {
    let paths = std::env::var("PATH")?;
    let path = paths
        .split(":")
        .map(|path| PathBuf::from(path).join(name))
        .find(|path| path.is_file())
        .with_context(|| format!("missing executable file command, {name}"))?;
    Ok(path)
}

fn command_and_args<'i>(input: ParseInput<'_, 'i>) -> ModalResult<(&'i str, Args)> {
    let (command, _, args) =
        (take_till(0.., AsChar::is_space), space0, parse_args).parse_next(input)?;
    Ok((command, args))
}

fn parse_args(input: ParseInput) -> ModalResult<Args> {
    let mut args: Vec<String> = repeat(0.., parse_arg).parse_next(input)?;
    args.retain(|s| !s.is_empty());
    Ok(Args(args))
}

fn parse_arg(input: ParseInput) -> ModalResult<String> {
    let mut arg = String::new();
    loop {
        let next = match input.as_bytes().first() {
            Some(b'\'') => parse_arg_single_quote.parse_next(input)?,
            Some(b'"') => &parse_arg_double_quote.parse_next(input)?,
            Some(c) if !c.is_space() => &parse_arg_no_quote.parse_next(input)?,
            _ => {
                multispace0.parse_next(input)?;
                if arg.is_empty() {
                    fail.parse_next(input)?;
                }
                break Ok(arg);
            }
        };
        arg.push_str(next);
    }
}

fn parse_arg_single_quote<'i>(input: ParseInput<'_, 'i>) -> ModalResult<&'i str> {
    delimited('\'', take_until(1.., "'"), '\'').parse_next(input)
}

fn parse_arg_no_quote(input: ParseInput) -> ModalResult<String> {
    let mut arg = String::new();
    loop {
        let next = take_till(0.., |c: char| " \t\\\'\"".contains(c)).parse_next(input)?;
        arg.push_str(next);

        if let Some(b'\\') = input.as_bytes().first() {
            match take(2usize).parse_next(input)? {
                "\\\n" => todo!(),
                c => arg.push(c.chars().last().unwrap()),
            }
        } else {
            break;
        }
    }

    Ok(arg)
}

fn parse_arg_double_quote(input: ParseInput) -> ModalResult<String> {
    delimited('"', parse_arg_double_quote_escape, '"').parse_next(input)
}

fn parse_arg_double_quote_escape(input: ParseInput) -> ModalResult<String> {
    let mut arg = String::new();
    loop {
        let next = take_till(0.., |c: char| "\"\\".contains(c)).parse_next(input)?;
        arg.push_str(next);

        if let Some(b'"') = input.as_bytes().first() {
            break Ok(arg);
        }

        match take(2usize).parse_next(input)? {
            "\\$" => arg.push('$'),
            "\\`" => arg.push('`'),
            "\\\"" => arg.push('"'),
            "\\\\" => arg.push('\\'),
            "\\\n" => todo!(),
            c => arg.push_str(c),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_command_and_args() {
        assert_eq!(
            command_and_args(&mut "echo 1 2 3"),
            Ok(("echo", Args(vec!["1".into(), "2".into(), "3".into()])))
        );
        assert_eq!(
            command_and_args(&mut "echo 1 2 3  "),
            Ok(("echo", Args(vec!["1".into(), "2".into(), "3".into()])))
        );
        assert_eq!(command_and_args(&mut "echo"), Ok(("echo", Args(vec![]))));
        assert_eq!(command_and_args(&mut ""), Ok(("", Args(vec![]))));
    }

    #[test]
    fn test_parse_arg() {
        assert_eq!(parse_arg(&mut "hello").unwrap(), "hello");
        assert_eq!(parse_arg(&mut "hello world").unwrap(), "hello");
        assert_eq!(parse_arg(&mut "'hello world'").unwrap(), "hello world");
        assert_eq!(parse_arg(&mut "'hello' world").unwrap(), "hello");
        assert_eq!(parse_arg(&mut "hello'world'").unwrap(), "helloworld");

        assert_eq!(parse_arg(&mut "\"hello world\"").unwrap(), "hello world");
        assert_eq!(parse_arg(&mut "\"hello\" world\"").unwrap(), "hello");
        assert_eq!(
            parse_arg(&mut "\"hello\\\" world\"").unwrap(),
            "hello\" world"
        );
        assert_eq!(
            parse_arg(&mut "\"hello\\$ world\"").unwrap(),
            "hello$ world"
        );
        assert_eq!(
            parse_arg(&mut "\"hello\\` world\"").unwrap(),
            "hello` world"
        );
        // assert_eq!(
        //     parse_arg(&mut "\"hello\\\n world\"").unwrap(),
        //     "hello\n world"
        // );
        assert_eq!(
            parse_arg(&mut "\"hello\\x world\"").unwrap(),
            "hello\\x world"
        );
        assert_eq!(parse_arg(&mut "\"hello\\$\"").unwrap(), "hello$");

        assert_eq!(parse_arg(&mut "hello\\ world").unwrap(), "hello world");

        assert_eq!(
            parse_arg(&mut "'hello\\\\world'").unwrap(),
            "hello\\\\world"
        );

        assert!(parse_arg(&mut " ").is_err())
    }

    #[test]
    fn test_parse_args() {
        assert_eq!(
            parse_args(&mut "hello").unwrap(),
            Args(vec!["hello".into()])
        );
        assert_eq!(
            parse_args(&mut "hello world").unwrap(),
            Args(vec!["hello".into(), "world".into()])
        );
        assert_eq!(
            parse_args(&mut "'hello' world").unwrap(),
            Args(vec!["hello".into(), "world".into()])
        );
        assert_eq!(
            parse_args(&mut "'hello world' hello world").unwrap(),
            Args(vec!["hello world".into(), "hello".into(), "world".into()])
        );
    }
}
