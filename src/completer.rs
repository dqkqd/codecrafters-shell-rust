use std::path::PathBuf;

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

        let mut pairs = vec![];
        pairs.extend_from_slice(&complete_builtin(&line[start..]));
        pairs.extend_from_slice(&complete_path(&line[start..]));

        Ok((start, pairs))
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

fn complete_path(word: &str) -> Vec<Pair> {
    match std::env::var("PATH") {
        Ok(path) => path
            .split(":")
            // construct absolute path
            .map(|p| format!("{}/{}*", p, word))
            // only find path that exists
            .filter_map(|pattern| glob::glob(&pattern).ok())
            // convert entries to path
            .flat_map(|entries| {
                let entries: Vec<PathBuf> =
                    entries.into_iter().filter_map(|entry| entry.ok()).collect();
                entries
            })
            // only use file name as executable file
            .filter_map(|p| {
                p.file_name()
                    .and_then(|f| f.to_str())
                    .map(|s| s.to_string())
            })
            .map(|executable| Pair {
                display: executable.clone(),
                replacement: executable + " ",
            })
            .collect(),
        Err(_) => vec![],
    }
}
