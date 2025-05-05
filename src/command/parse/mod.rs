use std::io;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{bail, Context};
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
struct CommandArg(String);

#[derive(Debug, PartialEq)]
enum RedirectArg {
    Output { n: i32, word: String },
    AppendOutput { n: i32, word: String },
    OutputAndError { word: String },
    AppendOutputAndError { word: String },
}

pub(super) type ParseInput<'a, 'b> = &'a mut &'b str;

pub(super) fn parse_command(input: ParseInput) -> anyhow::Result<Command> {
    match args.parse_next(input) {
        Ok((_redirect_args, command_args)) => {
            let (cmd, args) = command_args.split_first().unwrap();
            let cmd = cmd.0.to_string();
            let args = Args(args.iter().map(|v| v.0.to_string()).collect());
            let command = match BuiltinCommand::from_str(&cmd) {
                Ok(builtin) => InternalCommand::Builtin(builtin.with_args(args)),
                Err(_) => match path_lookup(&cmd) {
                    Ok(path) => InternalCommand::Path(PathCommand { path, args }),
                    Err(_) => InternalCommand::Invalid(InvalidCommand(cmd.to_string())),
                },
            };

            Ok(Command::new(
                PIn::Std(io::stdin()),
                POut::Std(io::stdout()),
                PErr::Std(io::stderr()),
                command,
            ))
        }
        Err(_) => bail!("cannot parse command `{}`", input),
    }
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

/// Helper struct to parse redirect and command, because winnow does not allow different types.
#[derive(Debug, PartialEq)]
enum RedirectOrCommand {
    Redirect(RedirectArg),
    Command(CommandArg),
}

fn args(input: ParseInput) -> ModalResult<(Vec<RedirectArg>, Vec<CommandArg>)> {
    let args: Vec<RedirectOrCommand> = repeat(
        1..,
        alt((
            raw_redirect_arg.map(RedirectOrCommand::Redirect),
            raw_command_arg.map(RedirectOrCommand::Command),
        )),
    )
    .parse_next(input)?;

    let mut redirect_args = vec![];
    let mut command_args = vec![];
    for arg in args {
        match arg {
            RedirectOrCommand::Redirect(redirect_arg) => redirect_args.push(redirect_arg),
            RedirectOrCommand::Command(command_arg) => command_args.push(command_arg),
        }
    }

    Ok((redirect_args, command_args))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn only_command_args() {
        assert_eq!(
            args(&mut "hello").unwrap(),
            (vec![], vec![CommandArg("hello".into())])
        );
        assert_eq!(
            args(&mut "hello world").unwrap(),
            (
                vec![],
                vec![CommandArg("hello".into()), CommandArg("world".into()),],
            )
        );
        assert_eq!(
            args(&mut "'hello' world").unwrap(),
            (
                vec![],
                vec![CommandArg("hello".into()), CommandArg("world".into()),],
            )
        );

        assert_eq!(
            args(&mut "'hello world' hello world").unwrap(),
            (
                vec![],
                vec![
                    CommandArg("hello world".into()),
                    CommandArg("hello".into()),
                    CommandArg("world".into()),
                ],
            )
        );
    }
}
