use std::{collections::HashSet, path::PathBuf};

use rustyline::{
    completion::{Candidate, Completer},
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

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompleteCandidate {
    pub display: String,
    pub replacement: String,
}
impl CompleteCandidate {
    fn new(s: &str) -> CompleteCandidate {
        CompleteCandidate {
            display: s.to_string(),
            replacement: s.to_string() + " ",
        }
    }
}

impl Candidate for CompleteCandidate {
    fn display(&self) -> &str {
        self.display.as_str()
    }

    fn replacement(&self) -> &str {
        self.replacement.as_str()
    }
}

pub(crate) struct ShellCompleter {}
impl Completer for ShellCompleter {
    type Candidate = CompleteCandidate;

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

        let pairs: HashSet<CompleteCandidate> = HashSet::from_iter(
            complete_builtin(word)
                .into_iter()
                .chain(complete_path(word)),
        );
        let mut pairs: Vec<CompleteCandidate> = pairs.into_iter().collect();
        pairs.sort();

        Ok((start, pairs))
    }
}

fn complete_builtin(word: &str) -> Vec<CompleteCandidate> {
    BuiltinCommand::iter()
        .filter_map(|command| {
            let command = command.as_ref();
            if command.starts_with(word) {
                Some(CompleteCandidate::new(command))
            } else {
                None
            }
        })
        .collect()
}

fn complete_path(word: &str) -> Vec<CompleteCandidate> {
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
                    .map(|s| CompleteCandidate::new(&s))
            })
            .collect(),
        Err(_) => vec![],
    }
}
