use std::io;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use command::raw_command_arg;
use redirect::raw_redirect_arg;
use winnow::{
    combinator::{alt, repeat},
    ModalResult, Parser,
};

use super::io::{PErr, PIn, POut};
use super::{Args, BuiltinCommand, Command, InternalCommand, InvalidCommand, PathCommand};

mod command;
mod redirect;

#[derive(Debug, PartialEq)]
enum Arg {
    Redirect(RedirectArg),
    Command(CommandArg),
}

#[derive(Debug, PartialEq)]
struct CommandArg(String);

#[derive(Debug, PartialEq)]
enum RedirectArg {
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
            Arg::Redirect(_) => None,
            Arg::Command(command) => Some(command.0),
        })
        .collect();
    let (command, args) = args.split_first().unwrap();
    Ok((command.to_string(), Args(args.to_vec())))
}

fn raw_args(input: ParseInput) -> ModalResult<Vec<Arg>> {
    repeat(
        1..,
        alt((
            raw_redirect_arg.map(Arg::Redirect),
            raw_command_arg.map(Arg::Command),
        )),
    )
    .parse_next(input)
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
    fn test_raw_command_args() {
        assert_eq!(
            raw_args(&mut "hello").unwrap(),
            [Arg::Command(CommandArg("hello".into()))],
        );
        assert_eq!(
            raw_args(&mut "hello world").unwrap(),
            [
                Arg::Command(CommandArg("hello".into())),
                Arg::Command(CommandArg("world".into())),
            ],
        );
        assert_eq!(
            raw_args(&mut "'hello' world").unwrap(),
            [
                Arg::Command(CommandArg("hello".into())),
                Arg::Command(CommandArg("world".into())),
            ],
        );

        assert_eq!(
            raw_args(&mut "'hello world' hello world").unwrap(),
            [
                Arg::Command(CommandArg("hello world".into())),
                Arg::Command(CommandArg("hello".into())),
                Arg::Command(CommandArg("world".into())),
            ],
        );
    }
}
