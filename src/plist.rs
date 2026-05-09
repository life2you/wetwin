use crate::error;
use anyhow::{Context, Result};
use std::path::Path;

const PLIST_BUDDY: &str = "/usr/libexec/PlistBuddy";

pub fn plist_buddy_path() -> &'static str {
    PLIST_BUDDY
}

pub fn bundle_identifier(plist_path: &Path) -> Result<String> {
    let plist = plist_path.display().to_string();
    error::run_command_capture(PLIST_BUDDY, &["-c", "Print :CFBundleIdentifier", &plist])
        .with_context(|| {
            format!(
                "Failed to read CFBundleIdentifier from {}",
                plist_path.display()
            )
        })
}

pub fn set_bundle_identifier(plist_path: &Path, bundle_id: &str) -> Result<()> {
    let plist = plist_path.display().to_string();
    let command = format!("Set :CFBundleIdentifier {bundle_id}");

    error::run_command_with_privilege_fallback(PLIST_BUDDY, &["-c", &command, &plist]).with_context(
        || {
            format!(
                "Failed to set CFBundleIdentifier to {bundle_id} in {}",
                plist_path.display()
            )
        },
    )
}
