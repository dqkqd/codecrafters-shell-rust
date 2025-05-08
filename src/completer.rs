use rustyline::{
    completion::{Completer, Pair},
    highlight::Highlighter,
    Completer, Context, Helper, Hinter, Validator,
};
use strum::IntoEnumIterator;

use crate::{command::BuiltinCommand, parse::StreamCommandParser};

#[derive(Helper, Completer, Hinter, Validator)]
pub(crate) struct ShellHelper {
    #[rustyline(Completer)]
    pub completer: ShellCompleter,
}

impl Highlighter for ShellHelper {}

pub(crate) struct ShellCompleter {}
impl Completer for ShellCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let _ = (line, pos, _ctx);

        let parser = StreamCommandParser::new(line);

        let remaining = parser.remaining().trim_start();
        let start = line.len() - remaining.len();
        let start = match remaining.chars().next() {
            Some('\"') | Some('\'') => start + 1, // TODO: auto replace quote
            _ => start,
        };

        let word = &line[start..];

        if word.is_empty() {
            return Ok((0, vec![]));
        }

        let builtin_pairs = complete_builtin(&line[start..]);
        Ok((start, builtin_pairs))
    }
}

fn complete_builtin(word: &str) -> Vec<Pair> {
    BuiltinCommand::iter()
        .filter_map(|command| {
            let command = command.as_ref();
            if command.starts_with(word) {
                Some(Pair {
                    display: command.into(),
                    replacement: command.to_string() + " ",
                })
            } else {
                None
            }
        })
        .collect()
}
