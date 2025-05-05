use std::io;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{bail, Context};
use command::command_arg;
use redirect::redirect_arg;
use winnow::{
    combinator::{alt, repeat},
    ModalResult, Parser,
};

use super::io::{PErr, PIn, POut, PType};
use super::{BuiltinCommand, Command, InternalCommand, InvalidCommand, PathCommand, ProgramArgs};

mod command;
mod redirect;

#[derive(Debug, PartialEq)]
struct CommandArg(String);

#[derive(Debug, PartialEq)]
enum RedirectArg {
    Output { n: i32, word: String },
    AppendOutput { n: i32, word: String },
}

pub(super) type ParseInput<'a, 'b> = &'a mut &'b str;

pub(crate) fn parse_command(input: ParseInput) -> anyhow::Result<Command> {
    match args.parse_next(input) {
        Ok((mut command_args, redirect_args)) => {
            if command_args.is_empty() {
                bail!("invalid command: `{input}`")
            }

            let redirect_pipes = redirect_args
                .into_iter()
                .map(|r| r.into_pipe())
                .collect::<anyhow::Result<Vec<_>>>()?;

            let mut stdin = vec![];
            let mut stdout = vec![];
            let mut stderr = vec![];
            for ptype in redirect_pipes {
                match ptype {
                    PType::In(pin) => stdin.push(pin),
                    PType::Out(pout) => stdout.push(pout),
                    PType::Err(perr) => stderr.push(perr),
                }
            }
            if stdin.is_empty() {
                stdin.push(PIn::Std(io::stdin()))
            }
            if stdout.is_empty() {
                stdout.push(POut::Std(io::stdout()))
            }
            if stderr.is_empty() {
                stderr.push(PErr::Std(io::stderr()))
            }

            let cmd = command_args.remove(0);

            let args = ProgramArgs(command_args.into_iter().map(|v| v.0).collect());
            let command = match BuiltinCommand::from_str(&cmd.0) {
                Ok(builtin) => InternalCommand::Builtin(builtin.with_args(args)),
                Err(_) => match path_lookup(&cmd.0) {
                    Ok(path) => InternalCommand::Path(PathCommand { path, args }),
                    Err(_) => InternalCommand::Invalid(InvalidCommand(cmd.0)),
                },
            };

            Ok(Command::new(stdin, stdout, stderr, command))
        }
        Err(_) => bail!("cannot parse command `{}`", input),
    }
}

pub(crate) fn path_lookup(name: &str) -> anyhow::Result<PathBuf> {
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

fn args(input: ParseInput) -> ModalResult<(Vec<CommandArg>, Vec<RedirectArg>)> {
    let args: Vec<RedirectOrCommand> = repeat(
        1..,
        alt((
            redirect_arg.map(RedirectOrCommand::Redirect),
            command_arg.map(RedirectOrCommand::Command),
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

    Ok((command_args, redirect_args))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn only_command_args() {
        assert_eq!(
            args(&mut "hello").unwrap(),
            (vec![CommandArg("hello".into())], vec![],)
        );
        assert_eq!(
            args(&mut "hello world").unwrap(),
            (
                vec![CommandArg("hello".into()), CommandArg("world".into()),],
                vec![],
            )
        );
        assert_eq!(
            args(&mut "'hello' world").unwrap(),
            (
                vec![CommandArg("hello".into()), CommandArg("world".into()),],
                vec![],
            )
        );

        assert_eq!(
            args(&mut "'hello world' hello world").unwrap(),
            (
                vec![
                    CommandArg("hello world".into()),
                    CommandArg("hello".into()),
                    CommandArg("world".into()),
                ],
                vec![],
            )
        );
    }

    #[test]
    fn only_redirect_args() {
        assert_eq!(
            args(&mut "> file").unwrap(),
            (
                vec![],
                vec![RedirectArg::Output {
                    n: 1,
                    word: "file".into()
                },],
            )
        );

        assert_eq!(
            args(&mut "2>|file").unwrap(),
            (
                vec![],
                vec![RedirectArg::Output {
                    n: 2,
                    word: "file".into()
                },],
            )
        );
    }

    #[test]
    fn command_args_and_redirect_args() {
        assert_eq!(
            args(&mut "echo > file").unwrap(),
            (
                vec![CommandArg("echo".into())],
                vec![RedirectArg::Output {
                    n: 1,
                    word: "file".into()
                },],
            )
        );

        assert_eq!(
            args(&mut "echo hello 2>|file").unwrap(),
            (
                vec![CommandArg("echo".into()), CommandArg("hello".into()),],
                vec![RedirectArg::Output {
                    n: 2,
                    word: "file".into()
                },],
            )
        );

        assert_eq!(
            args(&mut "echo hello >> file").unwrap(),
            (
                vec![CommandArg("echo".into()), CommandArg("hello".into()),],
                vec![RedirectArg::AppendOutput {
                    n: 1,
                    word: "file".into()
                },],
            )
        );

        assert_eq!(
            args(&mut "echo hello 2>> file").unwrap(),
            (
                vec![CommandArg("echo".into()), CommandArg("hello".into()),],
                vec![RedirectArg::AppendOutput {
                    n: 2,
                    word: "file".into()
                },],
            )
        );
    }
}
