use std::{
    env,
    io::{Read, Write},
    process::Stdio,
    str::FromStr,
};

use crate::{
    io::{write_stderr, write_stdout, PErr, PIn, POut},
    utils::path_lookup_exact,
};
use anyhow::Context;

use super::{BuiltinCommand, Command, InvalidCommand, PathCommand, ProgramArgs};

pub(super) trait Execute {
    fn execute(
        &mut self,
        stdin: &mut PIn,
        stdout: &mut [POut],
        stderr: &mut [PErr],
    ) -> anyhow::Result<()>;
}

impl Execute for Command {
    fn execute(
        &mut self,
        stdin: &mut PIn,
        stdout: &mut [POut],
        stderr: &mut [PErr],
    ) -> anyhow::Result<()> {
        match self {
            Command::Builtin(builtin_command) => builtin_command.execute(stdin, stdout, stderr),
            Command::Invalid(invalid_command) => invalid_command.execute(stdin, stdout, stderr),
            Command::Path(path_command) => path_command.execute(stdin, stdout, stderr),
        }
    }
}

impl Execute for InvalidCommand {
    fn execute(&mut self, _: &mut PIn, stdout: &mut [POut], _: &mut [PErr]) -> anyhow::Result<()> {
        write_stdout(
            stdout,
            format!("{}: command not found\n", self.0).as_bytes(),
        )?;

        Ok(())
    }
}

impl Execute for PathCommand {
    fn execute(
        &mut self,
        stdin: &mut PIn,
        stdout: &mut [POut],
        stderr: &mut [PErr],
    ) -> anyhow::Result<()> {
        let executable = self
            .path
            .file_name()
            .with_context(|| format!("invalid filename for path `{}`", self.path.display()))?;

        let mut command = std::process::Command::new(executable);
        let command = command.args(&self.args.0);

        let output = match stdin {
            PIn::File(file) => {
                let mut child = command
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?;

                let mut data = vec![];
                file.read_to_end(&mut data)?;
                let mut stdin = child.stdin.take().expect("failed to open stdin");
                std::thread::spawn(move || {
                    stdin.write_all(&data).expect("Failed to write to stdin");
                });
                child.wait_with_output()?
            }
            PIn::Empty => command.output()?,
        };

        write_stdout(stdout, &output.stdout)?;
        write_stderr(stderr, &output.stderr)?;

        Ok(())
    }
}

impl Execute for BuiltinCommand {
    fn execute(
        &mut self,
        _: &mut PIn,
        stdout: &mut [POut],
        stderr: &mut [PErr],
    ) -> anyhow::Result<()> {
        match self {
            BuiltinCommand::Exit(args) => exit_command(args, stderr),
            BuiltinCommand::Echo(args) => echo_command(args, stdout),
            BuiltinCommand::Type(args) => type_command(args, stdout),
            BuiltinCommand::Pwd => pwd_command(stdout),
            BuiltinCommand::Cd(args) => cd_command(args, stderr),
        }
    }
}

fn exit_command(args: &mut ProgramArgs, stderr: &mut [PErr]) -> anyhow::Result<()> {
    match args.0.first_mut() {
        Some(code) => match code.parse::<i32>() {
            Ok(code) => std::process::exit(code),
            Err(_) => {
                write_stderr(
                    stderr,
                    format!("invalid args: [{}]", args.0.join(",")).as_bytes(),
                )?;
                std::process::exit(-1);
            }
        },
        // no args given
        None => std::process::exit(0),
    }
}

fn echo_command(args: &mut ProgramArgs, stdout: &mut [POut]) -> anyhow::Result<()> {
    let mut iter = args.0.iter().peekable();
    while let Some(arg) = iter.next() {
        write_stdout(stdout, arg.as_bytes())?;
        if iter.peek().is_some() {
            write_stdout(stdout, b" ")?;
        }
    }
    write_stdout(stdout, b"\n")?;

    Ok(())
}

fn type_command(args: &mut ProgramArgs, stdout: &mut [POut]) -> anyhow::Result<()> {
    for arg in &args.0 {
        match BuiltinCommand::from_str(arg) {
            Ok(_) => write_stdout(stdout, format!("{arg} is a shell builtin\n").as_bytes())?,
            Err(_) => match path_lookup_exact(arg) {
                Ok(path) => write_stdout(
                    stdout,
                    format!("{arg} is {}\n", path.as_path().display()).as_bytes(),
                )?,
                Err(_) => write_stdout(stdout, format!("{arg}: not found\n").as_bytes())?,
            },
        }
    }

    Ok(())
}

fn pwd_command(stdout: &mut [POut]) -> anyhow::Result<()> {
    let current_dir = env::current_dir()?;
    write_stdout(
        stdout,
        format!("{}\n", current_dir.as_path().display()).as_bytes(),
    )?;

    Ok(())
}

fn cd_command(args: &mut ProgramArgs, stderr: &mut [PErr]) -> anyhow::Result<()> {
    match &args.0[..] {
        [path] => {
            let expanded_path = shellexpand::tilde(&path);
            if std::env::set_current_dir(expanded_path.as_ref()).is_err() {
                write_stderr(
                    stderr,
                    format!("cd: {path}: No such file or directory\n").as_bytes(),
                )?;
            }
        }
        _ => write_stderr(stderr, "cd: No path given".as_bytes())?,
    }

    Ok(())
}
