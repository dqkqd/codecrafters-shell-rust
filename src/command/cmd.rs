use std::{env, path::PathBuf, str::FromStr};

use super::{
    io::{PErr, PIn, POut},
    parse_cmd::{command_in_path, parse_i32},
    Execute,
};
use anyhow::Context;
use strum_macros::EnumString;

#[derive(Debug)]
pub(super) enum InternalCommand {
    Builtin(BuiltinCommand),
    Invalid(InvalidCommand),
    Path(PathCommand),
}

#[derive(Debug, Default, PartialEq)]
pub(super) struct Args(pub String);

#[derive(Debug, Default, PartialEq)]
pub(super) struct InvalidCommand(pub String);

#[derive(Debug, Default, PartialEq)]
pub(super) struct PathCommand {
    pub path: PathBuf,
    pub args: Args,
}

#[derive(Debug, PartialEq, EnumString)]
pub(super) enum BuiltinCommand {
    #[strum(serialize = "exit")]
    Exit(Args),
    #[strum(serialize = "echo")]
    Echo(Args),
    #[strum(serialize = "type")]
    Type(Args),
    #[strum(serialize = "pwd")]
    Pwd,
    #[strum(serialize = "cd")]
    Cd(Args),
}

impl Execute for InternalCommand {
    fn execute(
        &mut self,
        stdin: &mut PIn,
        stdout: &mut POut,
        stderr: &mut PErr,
    ) -> anyhow::Result<()> {
        match self {
            InternalCommand::Builtin(builtin_command) => {
                builtin_command.execute(stdin, stdout, stderr)
            }
            InternalCommand::Invalid(invalid_command) => {
                invalid_command.execute(stdin, stdout, stderr)
            }
            InternalCommand::Path(path_command) => path_command.execute(stdin, stdout, stderr),
        }
    }
}

impl Execute for InvalidCommand {
    fn execute(&mut self, _: &mut PIn, stdout: &mut POut, _: &mut PErr) -> anyhow::Result<()> {
        stdout.write_all_and_flush(format!("{}: command not found\n", self.0).as_bytes())?;
        Ok(())
    }
}

impl Execute for PathCommand {
    fn execute(&mut self, _: &mut PIn, stdout: &mut POut, stderr: &mut PErr) -> anyhow::Result<()> {
        let output = std::process::Command::new(
            self.path
                .file_name()
                .with_context(|| format!("invalid filename for path `{}`", self.path.display()))?,
        )
        .args(self.args.0.split_whitespace().collect::<Vec<_>>())
        .output()?;
        stdout.write_all_and_flush(&output.stdout)?;
        stderr.write_all_and_flush(&output.stderr)?;
        Ok(())
    }
}

impl Execute for BuiltinCommand {
    fn execute(&mut self, _: &mut PIn, stdout: &mut POut, stderr: &mut PErr) -> anyhow::Result<()> {
        match self {
            BuiltinCommand::Exit(args) => exit_command(args, stderr),
            BuiltinCommand::Echo(args) => echo_command(args, stdout),
            BuiltinCommand::Type(args) => type_command(args, stdout),
            BuiltinCommand::Pwd => pwd_command(stdout),
            BuiltinCommand::Cd(args) => cd_command(args, stderr),
        }
    }
}

fn exit_command(args: &mut Args, stderr: &mut PErr) -> anyhow::Result<()> {
    if let Ok(code) = parse_i32(&mut args.0.as_ref()) {
        std::process::exit(code)
    };
    stderr.write_all_and_flush(format!("invalid args: {}", args.0).as_bytes())?;
    Ok(())
}

fn echo_command(args: &mut Args, stdout: &mut POut) -> anyhow::Result<()> {
    let mut args = args.0.split_whitespace().peekable();
    while let Some(arg) = args.next() {
        stdout.write_all_and_flush(arg.as_bytes())?;
        if args.peek().is_some() {
            stdout.write_all_and_flush(b" ")?;
        }
    }
    stdout.write_all_and_flush(b"\n")?;
    Ok(())
}

fn type_command(args: &mut Args, stdout: &mut POut) -> anyhow::Result<()> {
    for arg in args.0.split_whitespace() {
        match BuiltinCommand::from_str(arg) {
            Ok(_) => {
                stdout.write_all_and_flush(format!("{arg} is a shell builtin\n").as_bytes())?
            }
            Err(_) => match command_in_path(arg) {
                Ok(path) => stdout.write_all_and_flush(
                    format!("{arg} is {}\n", path.as_path().display()).as_bytes(),
                )?,
                Err(_) => stdout.write_all_and_flush(format!("{arg}: not found\n").as_bytes())?,
            },
        }
    }
    Ok(())
}

fn pwd_command(stdout: &mut POut) -> anyhow::Result<()> {
    let current_dir = env::current_dir()?;
    stdout.write_all_and_flush(format!("{}\n", current_dir.as_path().display()).as_bytes())?;
    Ok(())
}

fn cd_command(args: &mut Args, stderr: &mut PErr) -> anyhow::Result<()> {
    let path = args.0.split_whitespace().next();
    if path.is_none_or(|path| std::env::set_current_dir(path).is_err()) {
        stderr.write_all_and_flush(
            format!(
                "cd: {}: No such file or directory\n",
                path.unwrap_or_default()
            )
            .as_bytes(),
        )?;
    }
    Ok(())
}
