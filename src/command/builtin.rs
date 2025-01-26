use anyhow::Result;

use crate::execute::{Execute, ExecutedOutput};

use super::{parse_tokens, Cmd};

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
                let (_, values) = parse_tokens(&typ)?;
                let command = Cmd::from_value_tokens(values)?;
                Ok(ExecBuiltinCmd::Type(Box::new(command)))
            }
        }
    }
}

impl Execute for ExecBuiltinCmd {
    fn execute(self) -> Result<ExecutedOutput> {
        let stdout = match self {
            ExecBuiltinCmd::Exit(code) => std::process::exit(code),
            ExecBuiltinCmd::Echo(echo) => echo,
            ExecBuiltinCmd::Pwd => {
                let current_directory = std::env::current_dir()?;
                current_directory.display().to_string()
            }
            ExecBuiltinCmd::Cd(directory) => {
                let expanded_directory = expand_home(&directory)?;
                if std::env::set_current_dir(&expanded_directory).is_err() {
                    format!("cd: {}: No such file or directory", directory)
                } else {
                    "".to_string()
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
                    format!("{} is a shell builtin", command_type)
                }
                Cmd::ExecFile(executable_file_command) => {
                    format!(
                        "{} is {}",
                        executable_file_command.command,
                        executable_file_command.path.display()
                    )
                }
                Cmd::Invalid(command) => format!("{}: not found", command),
            },
        };

        Ok(ExecutedOutput::new().with_stdout(stdout))
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
