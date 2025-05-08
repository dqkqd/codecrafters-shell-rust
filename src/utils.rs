use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;

pub(crate) fn path_lookup_exact(name: &str) -> Result<PathBuf> {
    let paths = std::env::var("PATH")?;
    let path = paths
        .split(":")
        .map(|path| PathBuf::from(path).join(name))
        .find(|path| path.is_file())
        .with_context(|| format!("missing executable file command, {name}"))?;
    Ok(path)
}
