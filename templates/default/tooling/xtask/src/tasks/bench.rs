use anyhow::Result;
use xshell::{Shell, cmd};

pub fn run(sh: &Shell) -> Result<()> {
    cmd!(sh, "cargo bench").run()?;
    Ok(())
}
