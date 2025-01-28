use std::{
    collections::BTreeSet,
    io::{StdoutLock, Write},
    path::PathBuf,
};

use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub(crate) enum TabCompletionState {
    NotPressed,
    Pressed,
}

/// Return the suffix
pub(crate) fn completed_suffix(
    stdout: &mut StdoutLock<'static>,
    pat: &str,
    state: TabCompletionState,
    raw: &str,
) -> Result<Option<String>> {
    let result = match completed_candidates(pat) {
        CompletedCandidates::None => {
            beep(stdout)?;
            stdout.flush()?;
            None
        }
        CompletedCandidates::One { candidate } => candidate
            .strip_prefix(pat)
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty()),

        CompletedCandidates::Multiple { prefix, candidates } => {
            match state {
                TabCompletionState::NotPressed => {
                    beep(stdout)?;
                }
                TabCompletionState::Pressed => {
                    stdout.write_all(b"\r\n")?;
                    let output = candidates.join("  ");
                    stdout.write_all(output.as_bytes())?;
                    stdout.write_all(b"\r\n$ ")?;

                    stdout.write_all(raw.as_bytes())?;
                    stdout.write_all(prefix.as_bytes())?;
                }
            }
            stdout.flush()?;
            None
        }
    };

    Ok(result)
}

enum CompletedCandidates {
    None,
    One {
        candidate: String,
    },
    Multiple {
        prefix: String,
        candidates: Vec<String>,
    },
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
        1 => CompletedCandidates::One {
            candidate: candidates.remove(0),
        },
        _ => {
            let longest_common_prefix = candidates.iter().fold(candidates[0].clone(), |acc, e| {
                longest_common_prefix(&acc, e)
            });

            CompletedCandidates::Multiple {
                prefix: longest_common_prefix,
                candidates,
            }
        }
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

fn beep<W: Write>(writer: &mut W) -> Result<()> {
    writer.write_all(b"\x07")?;
    Ok(())
}

fn longest_common_prefix(a: &str, b: &str) -> String {
    match a.chars().zip(b.chars()).position(|(x, y)| x != y) {
        Some(pos) => a[..pos].to_string(),
        None => {
            if a.len() > b.len() {
                b.to_string()
            } else {
                a.to_string()
            }
        }
    }
}
