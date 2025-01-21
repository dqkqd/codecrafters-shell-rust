mod builtin;
mod exec_file;

use anyhow::Result;
use builtin::{BuiltinCmd, ExecBuiltinCmd};
use exec_file::ExecFileCmd;

use crate::{
    error::CmdError,
    parser::{Parser, Token},
    Execute,
};

// TODO: lifetime
pub enum Cmd {
    Builtin(BuiltinCmd),
    ExecFileCmd(ExecFileCmd),
    Invalid(String),
}

impl TryFrom<String> for Cmd {
    type Error = CmdError;

    fn try_from(command: String) -> Result<Cmd, CmdError> {
        let parser = Parser::new(&command);
        let tokens = parser.into_tokens();
        let tokens = Token::to_string_no_whitespace(&tokens);

        let (command, args) = tokens.split_first().ok_or(CmdError::Empty)?;
        let remaining = args.join("");

        match command.as_str() {
            "exit" => Ok(Cmd::Builtin(BuiltinCmd::Exit(remaining))),
            "echo" => Ok(Cmd::Builtin(BuiltinCmd::Echo(remaining))),
            "pwd" => Ok(Cmd::Builtin(BuiltinCmd::Pwd)),
            "cd" => Ok(Cmd::Builtin(BuiltinCmd::Cd(remaining))),
            "type" => Ok(Cmd::Builtin(BuiltinCmd::Type(remaining))),
            _ => {
                if let Ok(exec_file_cmd) = ExecFileCmd::new(command.clone(), args.to_vec()) {
                    Ok(Cmd::ExecFileCmd(exec_file_cmd))
                } else {
                    Ok(Cmd::Invalid(command.clone()))
                }
            }
        }
    }
}

impl Execute for Cmd {
    fn execute(self) -> Result<()> {
        match self {
            Cmd::Builtin(builtin_command) => {
                let executable_command: ExecBuiltinCmd = builtin_command.try_into()?;
                executable_command.execute()?;
            }
            Cmd::ExecFileCmd(executable_file_command) => {
                executable_file_command.execute()?;
            }
            Cmd::Invalid(command) => println!("{}: command not found", command),
        }
        Ok(())
    }
}
