use anyhow::Result;

use crate::{error::CmdError, Execute};

use super::Cmd;

pub enum BuiltinCmd {
    Exit(String),
    Echo(String),
    Pwd,
    Cd(String),
    Type(String),
}

pub(super) enum ExecBuiltinCmd {
    Exit(i32),
    Echo(String),
    Pwd,
    Cd(String),
    Type(Box<Cmd>),
}

impl TryFrom<BuiltinCmd> for ExecBuiltinCmd {
    type Error = CmdError;

    fn try_from(command: BuiltinCmd) -> Result<ExecBuiltinCmd, CmdError> {
        match command {
            BuiltinCmd::Exit(code) => Ok(ExecBuiltinCmd::Exit(code.parse()?)),
            BuiltinCmd::Echo(echo) => Ok(ExecBuiltinCmd::Echo(echo)),
            BuiltinCmd::Pwd => Ok(ExecBuiltinCmd::Pwd),
            BuiltinCmd::Cd(directory) => Ok(ExecBuiltinCmd::Cd(directory)),
            BuiltinCmd::Type(typ) => {
                let command = Cmd::try_from(typ)?;
                Ok(ExecBuiltinCmd::Type(Box::new(command)))
            }
        }
    }
}

impl Execute for ExecBuiltinCmd {
    fn execute(self) -> Result<()> {
        match self {
            ExecBuiltinCmd::Exit(code) => std::process::exit(code),
            ExecBuiltinCmd::Echo(echo) => println!("{}", echo),
            ExecBuiltinCmd::Pwd => {
                let current_directory = std::env::current_dir()?;
                println!("{}", current_directory.display());
            }
            ExecBuiltinCmd::Cd(directory) => {
                let expanded_directory = expand_home(&directory)?;
                if std::env::set_current_dir(&expanded_directory).is_err() {
                    println!("cd: {}: No such file or directory", directory);
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
                    println!("{} is a shell builtin", command_type);
                }
                Cmd::ExecFileCmd(executable_file_command) => {
                    println!(
                        "{} is {}",
                        executable_file_command.command,
                        executable_file_command.path.display()
                    )
                }
                Cmd::Invalid(command) => println!("{}: not found", command),
            },
        }
        Ok(())
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
