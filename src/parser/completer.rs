use std::{
    collections::BTreeSet,
    io::{StdoutLock, Write},
    path::PathBuf,
};

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
    let mut candidates: Vec<String> = {
        let mut builtin_candidates: BTreeSet<String> = builtins()
            .iter()
            .filter(|s| s.starts_with(pat))
            .map(|s| s.to_string())
            .collect();

        if let Ok(path_candidates) = paths_candidates(pat) {
            builtin_candidates.extend(path_candidates);
        }
        builtin_candidates.into_iter().collect()
    };

    match candidates.len() {
        0 => CompletedCandidates::None,
        1 => CompletedCandidates::One(candidates.remove(0)),
        _ => CompletedCandidates::Multiple(candidates),
    }
}

fn builtins() -> Vec<&'static str> {
    vec!["exit", "pwd", "echo", "cd", "type"]
}

fn paths_candidates(pat: &str) -> Result<Vec<String>> {
    let path = std::env::var("PATH")?;
    let patterns: Vec<String> = path.split(":").map(|p| format!("{}/{}*", p, pat)).collect();

    let executables: Vec<String> = patterns
        .into_iter()
        .filter_map(|pattern| glob::glob(&pattern).ok())
        .flat_map(|entries| {
            let entries: Vec<PathBuf> =
                entries.into_iter().filter_map(|entry| entry.ok()).collect();
            entries
        })
        .filter_map(|p| {
            p.file_name()
                .and_then(|f| f.to_str())
                .map(|s| s.to_string())
        })
        .collect();

    Ok(executables)
}
