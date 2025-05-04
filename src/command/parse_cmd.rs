use std::io::{self};

use winnow::combinator::alt;
use winnow::token::rest;
use winnow::ModalResult;
use winnow::{
    ascii::{digit1, space1},
    combinator::preceded,
    Parser,
};

use crate::command::cmd::Builtin;

use super::cmd::{Command, InternalCommand};
use super::io::{PErr, POut};

type Input<'a, 'b> = &'a mut &'b str;

impl Command {
    pub(crate) fn parse(input: Input) -> Command {
        let command = alt((
            exit.map(|code| InternalCommand::Builtin(Builtin::Exit(code))),
            echo.map(|s| InternalCommand::Builtin(Builtin::Echo(s))),
            invalid_command.map(InternalCommand::Invalid),
        ))
        .parse_next(input)
        .unwrap_or_else(|_| panic!("cannot parse command {input}"));
        Command::new(POut::Std(io::stdout()), PErr::Std(io::stderr()), command)
    }
}

fn exit(input: Input) -> ModalResult<i32> {
    preceded(("exit", space1), digit1.try_map(str::parse)).parse_next(input)
}

fn invalid_command(input: Input) -> ModalResult<String> {
    rest.map(String::from).parse_next(input)
}

fn echo(input: Input) -> ModalResult<String> {
    preceded(("echo", space1), rest.map(String::from)).parse_next(input)
}
