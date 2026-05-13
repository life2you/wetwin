use crate::error;
use anyhow::{Context, Result};
use std::fs;
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

pub fn nested_bundle_plists(app_path: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut plists = Vec::new();
    collect_nested_bundle_plists(app_path, app_path, &mut plists)?;
    plists.sort();
    Ok(plists)
}

fn collect_nested_bundle_plists(
    current_dir: &Path,
    root_app_path: &Path,
    plists: &mut Vec<std::path::PathBuf>,
) -> Result<()> {
    for entry in fs::read_dir(current_dir)
        .with_context(|| format!("Failed to read directory {}", current_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let extension = path.extension().and_then(|value| value.to_str());
        if matches!(extension, Some("app" | "appex")) {
            if path != root_app_path {
                let plist_path = path.join("Contents/Info.plist");
                if plist_path.exists() {
                    plists.push(plist_path);
                }
            }
            collect_nested_bundle_plists(&path, root_app_path, plists)?;
            continue;
        }

        collect_nested_bundle_plists(&path, root_app_path, plists)?;
    }

    Ok(())
}
