use anyhow::Result;
use xshell::{Shell, cmd};

pub fn run(sh: &Shell) -> Result<()> {
    cmd!(sh, "cargo fmt --all --check").run()?;
    cmd!(sh, "cargo clippy --all-targets -- -D warnings").run()?;
    cmd!(sh, "cargo test").run()?;
    Ok(())
}
