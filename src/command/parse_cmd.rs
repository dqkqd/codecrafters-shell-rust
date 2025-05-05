use std::io;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use winnow::ascii::{digit1, space0};
use winnow::stream::AsChar;
use winnow::token::{rest, take_till};
use winnow::ModalResult;
use winnow::Parser;

use crate::command::cmd::BuiltinCommand;

use super::cmd::{Args, InternalCommand, InvalidCommand, PathCommand};
use super::io::{PErr, PIn, POut};
use super::Command;

pub(super) type ParseInput<'a, 'b> = &'a mut &'b str;

impl BuiltinCommand {
    fn with_args(self, args: Args) -> BuiltinCommand {
        match self {
            BuiltinCommand::Exit(_) => BuiltinCommand::Exit(args),
            BuiltinCommand::Echo(_) => BuiltinCommand::Echo(args),
            BuiltinCommand::Type(_) => BuiltinCommand::Type(args),
            BuiltinCommand::Pwd => BuiltinCommand::Pwd,
            BuiltinCommand::Cd(_) => BuiltinCommand::Cd(args),
        }
    }
}

pub(super) fn parse_command(input: ParseInput) -> Command {
    let (cmd, args) =
        command_and_args(input).unwrap_or_else(|_| panic!("cannot parse command {input}"));
    let args = Args(args.to_string());

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

fn command_and_args<'i>(input: ParseInput<'_, 'i>) -> ModalResult<(&'i str, &'i str)> {
    let (command, _, args) = (take_till(0.., AsChar::is_space), space0, rest).parse_next(input)?;
    Ok((command, args.trim_end()))
}

pub(super) fn parse_i32(input: ParseInput) -> ModalResult<i32> {
    digit1.try_map(str::parse).parse_next(input)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_command_and_args() {
        assert_eq!(command_and_args(&mut "echo 1 2 3"), Ok(("echo", "1 2 3")));
        assert_eq!(command_and_args(&mut "echo 1 2 3  "), Ok(("echo", "1 2 3")));
        assert_eq!(command_and_args(&mut "echo"), Ok(("echo", "")));
        assert_eq!(command_and_args(&mut ""), Ok(("", "")));
    }
}
