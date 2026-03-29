use anyhow::{Context, Result};
use std::process::{Command, Stdio};

pub fn restart_swayosd() -> Result<()> {
    let _ = Command::new("pkill").arg("swayosd-server").status();

    Command::new("swayosd-server")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start SwayOSD daemon")?;

    Ok(())
}
