use std::{
    env, fs,
    io::{BufRead, BufReader, Read},
    process::{Child, Stdio},
    str::FromStr,
    thread::JoinHandle,
};

use crate::{
    io::{write_stderr, write_stdout, PErr, PIn, POut},
    utils::path_lookup_exact,
    HIST_FILE,
};
use anyhow::{Context, Result};

use super::{BuiltinCommand, Command, CommandArgs, InvalidCommand, PathCommand};

#[derive(Debug)]
pub(crate) enum MaybeBlockedCommand {
    NonBlock,
    Block {
        stdout: JoinHandle<()>,
        stderr: JoinHandle<()>,
        child: Child,
    },
}

impl MaybeBlockedCommand {
    pub fn kill(self) -> Result<()> {
        match self {
            MaybeBlockedCommand::NonBlock => {}
            MaybeBlockedCommand::Block { mut child, .. } => {
                child.kill()?;
            }
        }

        Ok(())
    }

    pub fn wait(self) -> Result<()> {
        match self {
            MaybeBlockedCommand::NonBlock => {}
            MaybeBlockedCommand::Block {
                stdout,
                stderr,
                child,
                ..
            } => {
                stdout.join().expect("cannot join stdout");
                stderr.join().expect("cannot join stderr");
                child.wait_with_output()?;
            }
        }

        Ok(())
    }
}

pub(super) trait Execute {
    fn execute(
        &mut self,
        stdin: PIn,
        stdout: Vec<POut>,
        stderr: Vec<PErr>,
    ) -> Result<MaybeBlockedCommand>;
}

impl Execute for Command {
    fn execute(
        &mut self,
        stdin: PIn,
        stdout: Vec<POut>,
        stderr: Vec<PErr>,
    ) -> Result<MaybeBlockedCommand> {
        match self {
            Command::Builtin(builtin_command) => builtin_command.execute(stdin, stdout, stderr),
            Command::Invalid(invalid_command) => invalid_command.execute(stdin, stdout, stderr),
            Command::Path(path_command) => path_command.execute(stdin, stdout, stderr),
        }
    }
}

impl Execute for InvalidCommand {
    fn execute(
        &mut self,
        _: PIn,
        mut stdout: Vec<POut>,
        _: Vec<PErr>,
    ) -> Result<MaybeBlockedCommand> {
        write_stdout(
            &mut stdout,
            format!("{}: command not found\n", self.0).as_bytes(),
        )?;

        Ok(MaybeBlockedCommand::NonBlock)
    }
}

impl Execute for PathCommand {
    fn execute(
        &mut self,
        mut stdin: PIn,
        mut stdout: Vec<POut>,
        mut stderr: Vec<PErr>,
    ) -> Result<MaybeBlockedCommand> {
        let executable = self
            .path
            .file_name()
            .with_context(|| format!("invalid filename for path `{}`", self.path.display()))?;

        let mut command = std::process::Command::new(executable);
        let command = command.args(&self.args.0);
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let (cmd_stdin, cmd_stdout, cmd_stderr) = (
            child.stdin.take().with_context(|| "cannot get stdin")?,
            child.stdout.take().with_context(|| "cannot get stdout")?,
            child.stderr.take().with_context(|| "cannot get stderr")?,
        );

        std::thread::spawn(move || {
            stdin
                .send_to_writer(cmd_stdin)
                .expect("Failed to write to stdin");
        });
        let stdout = std::thread::spawn(move || {
            let mut reader = BufReader::new(cmd_stdout);
            let mut buf = [0; 1];
            while reader.read_exact(&mut buf).is_ok() {
                write_stdout(&mut stdout, &buf).expect("Failed to write to stdout");
            }
        });
        let stderr = std::thread::spawn(move || {
            let mut reader = BufReader::new(cmd_stderr);
            let mut buf = [0; 1];
            while reader.read_exact(&mut buf).is_ok() {
                write_stderr(&mut stderr, &buf).expect("Failed to write to stderr");
            }
        });

        Ok(MaybeBlockedCommand::Block {
            stdout,
            stderr,
            child,
        })
    }
}

impl Execute for BuiltinCommand {
    fn execute(
        &mut self,
        _: PIn,
        stdout: Vec<POut>,
        stderr: Vec<PErr>,
    ) -> Result<MaybeBlockedCommand> {
        match self {
            BuiltinCommand::Exit(args) => exit_command(args, stderr),
            BuiltinCommand::Echo(args) => echo_command(args, stdout),
            BuiltinCommand::Type(args) => type_command(args, stdout),
            BuiltinCommand::Pwd => pwd_command(stdout),
            BuiltinCommand::Cd(args) => cd_command(args, stderr),
            BuiltinCommand::History(args) => history_command(args, stdout, stderr),
        }
    }
}

fn exit_command(args: &mut CommandArgs, mut stderr: Vec<PErr>) -> Result<MaybeBlockedCommand> {
    match args.0.first_mut() {
        Some(code) => match code.parse::<i32>() {
            Ok(code) => std::process::exit(code),
            Err(_) => {
                write_stderr(
                    &mut stderr,
                    format!("invalid args: [{}]", args.0.join(",")).as_bytes(),
                )?;
                std::process::exit(-1);
            }
        },
        // no args given
        None => std::process::exit(0),
    }
}

fn echo_command(args: &mut CommandArgs, mut stdout: Vec<POut>) -> Result<MaybeBlockedCommand> {
    let mut iter = args.0.iter().peekable();
    while let Some(arg) = iter.next() {
        write_stdout(&mut stdout, arg.as_bytes())?;
        if iter.peek().is_some() {
            write_stdout(&mut stdout, b" ")?;
        }
    }
    write_stdout(&mut stdout, b"\n")?;

    Ok(MaybeBlockedCommand::NonBlock)
}

fn type_command(args: &mut CommandArgs, mut stdout: Vec<POut>) -> Result<MaybeBlockedCommand> {
    for arg in &args.0 {
        match BuiltinCommand::from_str(arg) {
            Ok(_) => write_stdout(
                &mut stdout,
                format!("{arg} is a shell builtin\n").as_bytes(),
            )?,
            Err(_) => match path_lookup_exact(arg) {
                Ok(path) => write_stdout(
                    &mut stdout,
                    format!("{arg} is {}\n", path.as_path().display()).as_bytes(),
                )?,
                Err(_) => write_stdout(&mut stdout, format!("{arg}: not found\n").as_bytes())?,
            },
        }
    }

    Ok(MaybeBlockedCommand::NonBlock)
}

fn pwd_command(mut stdout: Vec<POut>) -> Result<MaybeBlockedCommand> {
    let current_dir = env::current_dir()?;
    write_stdout(
        &mut stdout,
        format!("{}\n", current_dir.as_path().display()).as_bytes(),
    )?;

    Ok(MaybeBlockedCommand::NonBlock)
}

fn cd_command(args: &mut CommandArgs, mut stderr: Vec<PErr>) -> Result<MaybeBlockedCommand> {
    match &args.0[..] {
        [path] => {
            let expanded_path = shellexpand::tilde(&path);
            if std::env::set_current_dir(expanded_path.as_ref()).is_err() {
                write_stderr(
                    &mut stderr,
                    format!("cd: {path}: No such file or directory\n").as_bytes(),
                )?;
            }
        }
        _ => write_stderr(&mut stderr, "cd: No path given".as_bytes())?,
    }

    Ok(MaybeBlockedCommand::NonBlock)
}

fn history_command(
    args: &mut CommandArgs,
    mut stdout: Vec<POut>,
    mut stderr: Vec<PErr>,
) -> Result<MaybeBlockedCommand> {
    let mut hist_file = fs::File::open(HIST_FILE)?;
    let mut buf = vec![];
    hist_file.read_to_end(&mut buf)?;

    let mut skip = 0;
    if let Some(n) = args.0.first() {
        match n.parse::<usize>() {
            Ok(n) => {
                skip = buf.lines().count().saturating_sub(n);
            }
            Err(_) => {
                write_stderr(
                    &mut stderr,
                    format!("invalid limiting entries, not a number: {n}\n").as_bytes(),
                )?;
            }
        }
    }

    for (id, line) in buf.lines().enumerate().skip(skip) {
        let id = id + 1;
        let line = line?;
        write_stdout(&mut stdout, format!("    {id} {line}\n").as_bytes())?;
    }
    Ok(MaybeBlockedCommand::NonBlock)
}
