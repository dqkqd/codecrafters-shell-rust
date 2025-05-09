use std::io;
use std::str::FromStr;

use anyhow::{bail, Context};
use command::command_token;
use redirect::redirect_token;
use winnow::{
    ascii::multispace0,
    combinator::{alt, preceded},
    stream::{Offset, Stream as _},
    ModalResult, Parser, Partial,
};

use crate::{
    command::{BuiltinCommand, CommandArgs, InvalidCommand, PathCommand, StdioCommand},
    utils::path_lookup_exact,
};
use crate::{
    command::{Command, PipedCommand},
    io::{PErr, PIn, POut, PType},
};

mod command;
mod redirect;

#[derive(Debug, PartialEq, Eq)]
enum Token {
    Pipe,
    Redirect(RedirectToken),
    Command(CommandToken),
}

#[derive(Debug, PartialEq, Eq)]
struct CommandToken(String);

#[derive(Debug, PartialEq, Eq)]
enum RedirectToken {
    Input { n: i32, word: String },
    Output { n: i32, word: String },
    AppendOutput { n: i32, word: String },
}

pub(crate) type Stream<'i> = Partial<&'i str>;

#[derive(Debug)]
pub(crate) struct StreamCommandParser {
    remaining: String,
    parsed: Vec<(String, Token)>,
}

impl StreamCommandParser {
    pub fn new(line: &str) -> StreamCommandParser {
        let mut p = StreamCommandParser {
            remaining: line.to_string(),
            parsed: vec![],
        };
        p.parse();
        p
    }

    pub fn push(&mut self, s: &str) {
        self.remaining.push_str(s);
        self.parse();
    }

    pub fn is_empty(&self) -> bool {
        self.parsed.is_empty() && self.remaining.trim().is_empty()
    }

    pub fn finish(mut self) -> anyhow::Result<PipedCommand> {
        self.push("\n");

        let raw_input = self.input();

        // invalid
        if !self.remaining.trim().is_empty() {
            bail!("invalid command {}", &raw_input)
        }

        let mut tokens = vec![];
        let mut commands = vec![];

        let mut parsed = self.parsed;
        parsed.push(("".into(), Token::Pipe));
        for (_, arg) in parsed {
            match arg {
                Token::Pipe => {
                    let command = tokens_to_stdio_command(std::mem::take(&mut tokens))
                        .with_context(|| format!("invalid command: {}", &raw_input))?;
                    commands.push(command);
                }
                token => tokens.push(token),
            }
        }

        if commands.len() == 1 {
            Ok(PipedCommand::One(commands.pop().unwrap()))
        } else {
            Ok(PipedCommand::Many(commands))
        }
    }

    pub fn remaining(&self) -> &str {
        &self.remaining
    }

    fn input(&self) -> String {
        let mut input = String::new();
        for (p, _) in &self.parsed {
            input += p;
        }
        input += self.remaining();
        input
    }

    fn parse(&mut self) {
        loop {
            let mut stream = Stream::new(&self.remaining);
            let start = stream.checkpoint();
            match token.parse_next(&mut stream) {
                Ok(tok) => {
                    let end = stream.offset_from(&start);

                    let mut parsed_input = self.remaining.split_off(end);
                    std::mem::swap(&mut self.remaining, &mut parsed_input);
                    self.parsed.push((parsed_input, tok));
                }
                Err(_) => break,
            }
        }
    }
}

fn token(stream: &mut Stream) -> ModalResult<Token> {
    alt((
        preceded(multispace0, "|").map(|_| Token::Pipe),
        redirect_token.map(Token::Redirect),
        command_token.map(Token::Command),
    ))
    .parse_next(stream)
}

fn tokens_to_stdio_command(tokens: Vec<Token>) -> anyhow::Result<StdioCommand> {
    let mut redirect_args = vec![];
    let mut command_args = vec![];
    for tok in tokens {
        match tok {
            Token::Pipe => {}
            Token::Redirect(redirect_arg) => redirect_args.push(redirect_arg),
            Token::Command(command_arg) => command_args.push(command_arg),
        }
    }

    if command_args.is_empty() {
        bail!("no command args")
    }

    let redirect_pipes = redirect_args
        .into_iter()
        .map(|r| r.into_pipe())
        .collect::<anyhow::Result<Vec<_>>>()?;

    let mut stdin = vec![];
    let mut stdout = vec![];
    let mut stderr = vec![];
    for ptype in redirect_pipes {
        match ptype {
            PType::In(pin) => stdin.push(pin),
            PType::Out(pout) => stdout.push(pout),
            PType::Err(perr) => stderr.push(perr),
        }
    }
    let stdin = stdin.pop().unwrap_or(PIn::Empty);
    if stdout.is_empty() {
        stdout.push(POut::Std(io::stdout()))
    }
    if stderr.is_empty() {
        stderr.push(PErr::Std(io::stderr()))
    }

    let cmd = command_args.remove(0);

    let args = CommandArgs(command_args.into_iter().map(|v| v.0).collect());
    let command = match BuiltinCommand::from_str(&cmd.0) {
        Ok(builtin) => Command::Builtin(builtin.with_args(args)),
        Err(_) => match path_lookup_exact(&cmd.0) {
            Ok(path) => Command::Path(PathCommand { path, args }),
            Err(_) => Command::Invalid(InvalidCommand(cmd.0)),
        },
    };

    Ok(StdioCommand::new(stdin, stdout, stderr, command))
}

#[cfg(test)]
mod test {

    use super::*;

    fn parser(command: &str) -> StreamCommandParser {
        StreamCommandParser::new(&format!("{command}\n"))
    }

    #[test]
    fn test_tokens_only_command() {
        assert_eq!(
            parser("hello").parsed,
            vec![("hello".into(), Token::Command(CommandToken("hello".into())))],
        );
        assert_eq!(
            parser("hello world").parsed,
            vec![
                ("hello".into(), Token::Command(CommandToken("hello".into()))),
                (
                    " world".into(),
                    Token::Command(CommandToken("world".into()))
                )
            ],
        );
        assert_eq!(
            parser("'hello' world").parsed,
            vec![
                (
                    "'hello'".into(),
                    Token::Command(CommandToken("hello".into()))
                ),
                (
                    " world".into(),
                    Token::Command(CommandToken("world".into()))
                )
            ],
        );
        assert_eq!(
            parser("'hello world' hello world").parsed,
            vec![
                (
                    "'hello world'".into(),
                    Token::Command(CommandToken("hello world".into()))
                ),
                (
                    " hello".into(),
                    Token::Command(CommandToken("hello".into()))
                ),
                (
                    " world".into(),
                    Token::Command(CommandToken("world".into()))
                )
            ],
        );
    }

    #[test]
    fn test_tokens_only_redirect() {
        assert_eq!(
            parser("> file").parsed,
            vec![(
                "> file".into(),
                Token::Redirect(RedirectToken::Output {
                    n: 1,
                    word: "file".into()
                })
            )],
        );

        assert_eq!(
            parser("2>|file").parsed,
            vec![(
                "2>|file".into(),
                Token::Redirect(RedirectToken::Output {
                    n: 2,
                    word: "file".into()
                })
            )],
        );
    }

    #[test]
    fn command_args_and_redirect_args() {
        assert_eq!(
            parser("echo > file").parsed,
            vec![
                ("echo".into(), Token::Command(CommandToken("echo".into()))),
                (
                    " > file".into(),
                    Token::Redirect(RedirectToken::Output {
                        n: 1,
                        word: "file".into()
                    })
                )
            ],
        );
        assert_eq!(
            parser("echo hello 2>|file").parsed,
            vec![
                ("echo".into(), Token::Command(CommandToken("echo".into()))),
                (
                    " hello".into(),
                    Token::Command(CommandToken("hello".into()))
                ),
                (
                    " 2>|file".into(),
                    Token::Redirect(RedirectToken::Output {
                        n: 2,
                        word: "file".into()
                    })
                )
            ],
        );
        assert_eq!(
            parser("echo hello >> file").parsed,
            vec![
                ("echo".into(), Token::Command(CommandToken("echo".into()))),
                (
                    " hello".into(),
                    Token::Command(CommandToken("hello".into()))
                ),
                (
                    " >> file".into(),
                    Token::Redirect(RedirectToken::AppendOutput {
                        n: 1,
                        word: "file".into()
                    })
                )
            ],
        );
        assert_eq!(
            parser("echo hello 2>> file").parsed,
            vec![
                ("echo".into(), Token::Command(CommandToken("echo".into()))),
                (
                    " hello".into(),
                    Token::Command(CommandToken("hello".into()))
                ),
                (
                    " 2>> file".into(),
                    Token::Redirect(RedirectToken::AppendOutput {
                        n: 2,
                        word: "file".into()
                    })
                )
            ],
        );
    }

    #[test]
    fn pipe() {
        assert_eq!(
            parser("one | two").parsed,
            vec![
                ("one".into(), Token::Command(CommandToken("one".into()))),
                (" |".into(), Token::Pipe),
                (" two".into(), Token::Command(CommandToken("two".into()))),
            ]
        )
    }
}
