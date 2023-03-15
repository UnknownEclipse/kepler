use std::process::Command;

use color_eyre::{eyre::bail, Result};

pub fn build(options: BuildOptions<'_>) -> Result<()> {
    let mut command = Command::new("cargo");

    command.arg("+nightly").arg("build");
    command
        .arg("-Z")
        .arg("build-std=core,alloc,compiler_builtins")
        .arg("--target")
        .arg(options.target);

    if options.release {
        command.arg("--release");
    }
    command.arg("--bin").arg("kernel");

    dbg!(&command);

    let mut child = command.spawn()?;
    let exit_code = child.wait()?;
    if !exit_code.success() {
        bail!("build finished with non-zero exit code");
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct BuildOptions<'a> {
    pub target: &'a str,
    pub release: bool,
    pub package: Option<&'a str>,
}
