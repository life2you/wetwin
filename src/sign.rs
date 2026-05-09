use crate::error;
use anyhow::{Context, Result};
use std::path::Path;

pub fn clear_quarantine(app_path: &Path) -> Result<()> {
    let app = app_path.display().to_string();
    error::run_command_with_privilege_fallback("xattr", &["-cr", &app]).with_context(|| {
        format!(
            "Failed to clear quarantine attributes for {}",
            app_path.display()
        )
    })
}

pub fn ad_hoc_sign(app_path: &Path) -> Result<()> {
    let app = app_path.display().to_string();
    error::run_command_with_privilege_fallback(
        "codesign",
        &["--force", "--deep", "--sign", "-", &app],
    )
    .with_context(|| format!("Failed to ad-hoc sign {}", app_path.display()))
}
