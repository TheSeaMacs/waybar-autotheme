use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn update_rofi_colors(path: &Path, bg: &str, fg: &str) -> Result<()> {
    let content = format!(
        "* {{\n    background: {};\n    foreground: {};\n}}\n",
        bg, fg
    );
    fs::write(path, content).context("Failed to write Rofi colors")?;
    Ok(())
}
