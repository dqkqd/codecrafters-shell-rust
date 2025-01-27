use std::io::{StdoutLock, Write};

use anyhow::Result;

/// Return the suffix
pub(crate) fn completed_suffix(
    stdout: &mut StdoutLock<'static>,
    pat: &str,
) -> Result<Option<String>> {
    let completed_candidates = completed_candidates(pat);
    let result = match completed_candidates.len() {
        0 => None,
        1 => completed_candidates[0]
            .strip_prefix(pat)
            .map(|s| s.to_string()),
        _ => {
            stdout.write_all(b"\r\n")?;
            for s in completed_candidates {
                stdout.write_all(s.as_bytes())?;
                stdout.write_all(b"\r\n")?;
            }
            stdout.flush()?;
            None
        }
    };

    Ok(result)
}

fn completed_candidates(pat: &str) -> Vec<String> {
    builtins()
        .iter()
        .filter(|s| s.starts_with(pat))
        .map(|s| s.to_string())
        .collect()
}

fn builtins() -> Vec<&'static str> {
    vec!["exit", "pwd", "echo", "cd", "type"]
}
