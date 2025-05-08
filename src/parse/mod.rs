use std::io;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{bail, Context};
use command::command_token;
use redirect::redirect_token;
use winnow::{
    combinator::alt,
    stream::{Offset, Stream as _},
    ModalResult, Parser, Partial,
};

use crate::command::{
    BuiltinCommand, Command, InvalidCommand, PathCommand, PipedCommand, ProgramArgs,
};
use crate::io::{PErr, PIn, POut, PType};

mod command;
mod redirect;

/// Helper struct to parse redirect and command, because winnow does not allow different types.
#[derive(Debug, PartialEq, Eq)]
enum Token {
    Redirect(RedirectToken),
    Command(CommandToken),
}

#[derive(Debug, PartialEq, Eq)]
struct CommandToken(String);

#[derive(Debug, PartialEq, Eq)]
enum RedirectToken {
    Output { n: i32, word: String },
    AppendOutput { n: i32, word: String },
}

pub(crate) type Stream<'i> = Partial<&'i str>;

#[derive(Debug, Default)]
pub(crate) struct StreamCommandParser {
    raw_input: String,
    parsed: Vec<(String, Token)>,
}

impl StreamCommandParser {
    pub fn new() -> StreamCommandParser {
        StreamCommandParser::default()
    }

    pub fn push(&mut self, s: &str) {
        self.raw_input.push_str(s);
        self.parse();
    }

    pub fn is_empty(&self) -> bool {
        self.parsed.is_empty() && self.raw_input.trim().is_empty()
    }

    pub fn finish(mut self) -> anyhow::Result<PipedCommand> {
        self.push("\n");

        let raw_input = self.raw_input();

        // invalid
        if !self.raw_input.trim().is_empty() {
            bail!("invalid command {}", &raw_input)
        }

        let mut redirect_args = vec![];
        let mut command_args = vec![];
        for (_, arg) in self.parsed {
            match arg {
                Token::Redirect(redirect_arg) => redirect_args.push(redirect_arg),
                Token::Command(command_arg) => command_args.push(command_arg),
            }
        }

        if command_args.is_empty() {
            bail!("invalid command: {}", &raw_input)
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
        if stdin.is_empty() {
            stdin.push(PIn::Std(io::stdin()))
        }
        if stdout.is_empty() {
            stdout.push(POut::Std(io::stdout()))
        }
        if stderr.is_empty() {
            stderr.push(PErr::Std(io::stderr()))
        }

        let cmd = command_args.remove(0);

        let args = ProgramArgs(command_args.into_iter().map(|v| v.0).collect());
        let command = match BuiltinCommand::from_str(&cmd.0) {
            Ok(builtin) => Command::Builtin(builtin.with_args(args)),
            Err(_) => match path_lookup(&cmd.0) {
                Ok(path) => Command::Path(PathCommand { path, args }),
                Err(_) => Command::Invalid(InvalidCommand(cmd.0)),
            },
        };

        Ok(PipedCommand::new(stdin, stdout, stderr, command))
    }

    fn raw_input(&self) -> String {
        let mut input = String::new();
        for (p, _) in &self.parsed {
            input += p;
        }
        input += &self.raw_input;
        input
    }

    fn parse(&mut self) {
        loop {
            let mut stream = Stream::new(&self.raw_input);
            let start = stream.checkpoint();
            match token.parse_next(&mut stream) {
                Ok(tok) => {
                    let end = stream.offset_from(&start);

                    let mut parsed_input = self.raw_input.split_off(end);
                    std::mem::swap(&mut self.raw_input, &mut parsed_input);
                    self.parsed.push((parsed_input, tok));
                }
                Err(_) => break,
            }
        }
    }
}

pub(crate) fn path_lookup(name: &str) -> anyhow::Result<PathBuf> {
    let paths = std::env::var("PATH")?;
    let path = paths
        .split(":")
        .map(|path| PathBuf::from(path).join(name))
        .find(|path| path.is_file())
        .with_context(|| format!("missing executable file command, {name}"))?;
    Ok(path)
}

fn token(stream: &mut Stream) -> ModalResult<Token> {
    alt((
        redirect_token.map(Token::Redirect),
        command_token.map(Token::Command),
    ))
    .parse_next(stream)
}

#[cfg(test)]
mod test {

    use super::*;

    fn parser(command: &str) -> StreamCommandParser {
        let mut p = StreamCommandParser::default();
        p.push(command);
        p.push("\n");
        p
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
}
