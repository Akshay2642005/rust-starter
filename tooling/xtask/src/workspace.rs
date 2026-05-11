use std::path::PathBuf;

use anyhow::{Context, Result};
use xshell::Shell;

pub fn root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .context("failed to resolve workspace root")
}

pub fn shell() -> Result<Shell> {
    let sh = Shell::new().context("failed to create shell")?;
    sh.change_dir(root()?);
    Ok(sh)
}
