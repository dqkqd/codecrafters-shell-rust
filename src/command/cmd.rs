use std::str::FromStr;

use super::{
    io::{PErr, PIn, POut},
    parse_cmd::parse_i32,
    Execute,
};
use strum_macros::EnumString;

pub(super) enum InternalCommand {
    Builtin(BuiltinCommand),
    Invalid(InvalidCommand),
}

#[derive(Debug, Default, PartialEq)]
pub(super) struct Args(pub String);

#[derive(Debug, Default, PartialEq)]
pub(super) struct InvalidCommand(pub String);

#[derive(Debug, PartialEq, EnumString)]
pub(super) enum BuiltinCommand {
    #[strum(serialize = "exit")]
    Exit(Args),
    #[strum(serialize = "echo")]
    Echo(Args),
    #[strum(serialize = "type")]
    Type(Args),
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
        }
    }
}

impl Execute for InvalidCommand {
    fn execute(&mut self, _: &mut PIn, stdout: &mut POut, _: &mut PErr) -> anyhow::Result<()> {
        stdout.write_all_and_flush(&format!("{}: command not found\n", self.0))?;
        Ok(())
    }
}

impl Execute for BuiltinCommand {
    fn execute(&mut self, _: &mut PIn, stdout: &mut POut, stderr: &mut PErr) -> anyhow::Result<()> {
        match self {
            BuiltinCommand::Exit(args) => exit_command(args, stderr),
            BuiltinCommand::Echo(args) => echo_command(args, stdout),
            BuiltinCommand::Type(args) => type_command(args, stdout),
        }
    }
}

fn exit_command(args: &mut Args, stderr: &mut PErr) -> anyhow::Result<()> {
    if let Ok(code) = parse_i32(&mut args.0.as_ref()) {
        std::process::exit(code)
    };
    stderr.write_all_and_flush(&format!("invalid args: {}", args.0))?;
    Ok(())
}

fn echo_command(args: &mut Args, stdout: &mut POut) -> anyhow::Result<()> {
    let mut args = args.0.split_whitespace().peekable();
    while let Some(arg) = args.next() {
        stdout.write_all_and_flush(arg)?;
        if args.peek().is_some() {
            stdout.write_all_and_flush(" ")?;
        }
    }
    stdout.write_all_and_flush("\n")?;
    Ok(())
}

fn type_command(args: &mut Args, stdout: &mut POut) -> anyhow::Result<()> {
    for arg in args.0.split_whitespace() {
        match BuiltinCommand::from_str(arg) {
            Ok(_) => stdout.write_all_and_flush(&format!("{arg} is a shell builtin\n"))?,
            Err(_) => stdout.write_all_and_flush(&format!("{arg}: not found\n"))?,
        }
    }
    Ok(())
}
