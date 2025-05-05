use std::{env, str::FromStr};

use super::{
    io::{PErr, PIn, POut},
    parse::path_lookup,
    Args, BuiltinCommand, InternalCommand, InvalidCommand, PathCommand,
};
use anyhow::Context;

pub(super) trait Execute {
    fn execute(
        &mut self,
        stdin: &mut PIn,
        stdout: &mut POut,
        stderr: &mut PErr,
    ) -> anyhow::Result<()>;
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
        .args(&self.args.0)
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
    match args.0.first_mut() {
        Some(code) => match code.parse::<i32>() {
            Ok(code) => std::process::exit(code),
            Err(_) => {
                stderr.write_all_and_flush(
                    format!("invalid args: [{}]", args.0.join(",")).as_bytes(),
                )?;
                std::process::exit(-1);
            }
        },
        // no args given
        None => std::process::exit(0),
    }
}

fn echo_command(args: &mut Args, stdout: &mut POut) -> anyhow::Result<()> {
    let mut iter = args.0.iter().peekable();
    while let Some(arg) = iter.next() {
        stdout.write_all_and_flush(arg.as_bytes())?;
        if iter.peek().is_some() {
            stdout.write_all_and_flush(b" ")?;
        }
    }
    stdout.write_all_and_flush(b"\n")?;
    Ok(())
}

fn type_command(args: &mut Args, stdout: &mut POut) -> anyhow::Result<()> {
    for arg in &args.0 {
        match BuiltinCommand::from_str(arg) {
            Ok(_) => {
                stdout.write_all_and_flush(format!("{arg} is a shell builtin\n").as_bytes())?
            }
            Err(_) => match path_lookup(arg) {
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
    match &args.0[..] {
        [path] => {
            let expanded_path = shellexpand::tilde(&path);
            if std::env::set_current_dir(expanded_path.as_ref()).is_err() {
                stderr.write_all_and_flush(
                    format!("cd: {path}: No such file or directory\n").as_bytes(),
                )?;
            }
        }
        _ => {
            stderr.write_all_and_flush("cd: No path given".as_bytes())?;
        }
    }
    Ok(())
}
