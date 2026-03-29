use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn update_css_colors(path: &Path, bg: &str, fg: &str) -> Result<()> {
    let content = format!(
        "@define-color background {};\n@define-color text-foreground {};\n",
        bg, fg
    );
    fs::write(path, content).context("Failed to write CSS colors")?;
    Ok(())
}
