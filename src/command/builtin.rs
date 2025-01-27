use anyhow::Result;

use crate::parser::parse_tokens;

use super::{Cmd, Execute, ExecutedOutput};

pub(crate) enum BuiltinCmd {
    Exit(String),
    Echo(String),
    Pwd,
    Cd(String),
    Type(String),
}

pub(crate) enum ExecBuiltinCmd {
    Exit(i32),
    Echo(String),
    Pwd,
    Cd(String),
    Type(Box<Cmd>),
}

impl BuiltinCmd {
    pub fn into_exec(self) -> Result<ExecBuiltinCmd> {
        match self {
            BuiltinCmd::Exit(code) => Ok(ExecBuiltinCmd::Exit(code.parse()?)),
            BuiltinCmd::Echo(echo) => Ok(ExecBuiltinCmd::Echo(echo)),
            BuiltinCmd::Pwd => Ok(ExecBuiltinCmd::Pwd),
            BuiltinCmd::Cd(directory) => Ok(ExecBuiltinCmd::Cd(directory)),
            BuiltinCmd::Type(typ) => {
                let (_, values) =
                    parse_tokens(typ.split_whitespace().map(|s| s.to_string()).collect())?;
                let command = Cmd::from_value_tokens(values)?;
                Ok(ExecBuiltinCmd::Type(Box::new(command)))
            }
        }
    }
}

impl Execute for ExecBuiltinCmd {
    fn execute(self) -> Result<ExecutedOutput> {
        let output = ExecutedOutput::new();
        let output = match self {
            ExecBuiltinCmd::Exit(code) => std::process::exit(code),
            ExecBuiltinCmd::Echo(echo) => output.with_stdout(&echo),
            ExecBuiltinCmd::Pwd => {
                let current_directory = std::env::current_dir()?;
                ExecutedOutput::new().with_stdout(current_directory.display().to_string())
            }
            ExecBuiltinCmd::Cd(directory) => {
                let expanded_directory = expand_home(&directory)?;
                if std::env::set_current_dir(&expanded_directory).is_err() {
                    output.with_stderr(format!("cd: {}: No such file or directory", directory))
                } else {
                    output.with_stdout("")
                }
            }
            ExecBuiltinCmd::Type(typ) => match *typ {
                Cmd::Builtin(builtin_command) => {
                    let command_type = match builtin_command {
                        BuiltinCmd::Exit(_) => "exit",
                        BuiltinCmd::Echo(_) => "echo",
                        BuiltinCmd::Pwd => "pwd",
                        BuiltinCmd::Cd(_) => "cd",
                        BuiltinCmd::Type(_) => "type",
                    };
                    output.with_stdout(format!("{} is a shell builtin", command_type))
                }
                Cmd::ExecFile(executable_file_command) => output.with_stdout(format!(
                    "{} is {}",
                    executable_file_command.command,
                    executable_file_command.path.display()
                )),
                Cmd::Invalid(command) => output.with_stderr(format!("{}: not found", command)),
            },
        };

        Ok(output)
    }
}

fn expand_home(path: &str) -> Result<String> {
    let mut p = std::env::var("HOME")?;
    if !p.ends_with("/") {
        p.push('/');
    }
    let expanded = path.replace("~", &p);

    Ok(expanded)
}
