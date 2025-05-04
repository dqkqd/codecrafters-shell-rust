use std::io::{self};

use winnow::combinator::alt;
use winnow::token::rest;
use winnow::ModalResult;
use winnow::{
    ascii::{digit1, space1},
    combinator::preceded,
    Parser,
};

use crate::command::cmd::BuiltinCommand;

use super::cmd::{Command, CommandWithPipe};
use super::io::{PErr, POut};

type Input<'a, 'b> = &'a mut &'b str;

impl CommandWithPipe {
    pub(crate) fn parse(input: Input) -> CommandWithPipe {
        let command = alt((
            exit.map(|code| Command::Builtin(BuiltinCommand::ExitCommand(code))),
            invalid_command.map(Command::InvalidCommand),
        ))
        .parse_next(input)
        .unwrap_or_else(|_| panic!("cannot parse command {input}"));
        CommandWithPipe::new(POut::Std(io::stdout()), PErr::Std(io::stderr()), command)
    }
}

fn exit(input: Input) -> ModalResult<i32> {
    preceded(("exit", space1), digit1.try_map(str::parse)).parse_next(input)
}

fn invalid_command(input: Input) -> ModalResult<String> {
    rest.map(|s: &str| s.to_string()).parse_next(input)
}
