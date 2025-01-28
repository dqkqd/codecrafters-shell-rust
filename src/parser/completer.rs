use std::io::{StdoutLock, Write};

use anyhow::Result;

/// Return the suffix
pub(crate) fn completed_suffix(
    stdout: &mut StdoutLock<'static>,
    pat: &str,
) -> Result<Option<String>> {
    let result = match completed_candidates(pat) {
        CompletedCandidates::None => {
            stdout.write_all(b"\x07")?;
            stdout.flush()?;
            None
        }
        CompletedCandidates::One(candidate) => candidate.strip_prefix(pat).map(|s| s.to_string()),
        CompletedCandidates::Multiple(candidates) => {
            stdout.write_all(b"\r\n")?;
            for s in candidates {
                stdout.write_all(s.as_bytes())?;
                stdout.write_all(b"\r\n")?;
            }
            stdout.flush()?;
            None
        }
    };

    Ok(result)
}

enum CompletedCandidates {
    None,
    One(String),
    Multiple(Vec<String>),
}

fn completed_candidates(pat: &str) -> CompletedCandidates {
    let mut candidates: Vec<String> = builtins()
        .iter()
        .filter(|s| s.starts_with(pat))
        .map(|s| s.to_string())
        .collect();

    match candidates.len() {
        0 => CompletedCandidates::None,
        1 => CompletedCandidates::One(candidates.remove(0)),
        _ => CompletedCandidates::Multiple(candidates),
    }
}

fn builtins() -> Vec<&'static str> {
    vec!["exit", "pwd", "echo", "cd", "type"]
}
