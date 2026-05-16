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
    let command = format!(
        "Set :CFBundleIdentifier {}",
        plist_string_literal(bundle_id)
    );

    error::run_command_with_privilege_fallback(PLIST_BUDDY, &["-c", &command, &plist]).with_context(
        || {
            format!(
                "Failed to set CFBundleIdentifier to {bundle_id} in {}",
                plist_path.display()
            )
        },
    )
}

pub fn set_display_name(plist_path: &Path, display_name: &str) -> Result<()> {
    set_string_key(plist_path, "CFBundleDisplayName", display_name)?;
    set_string_key(plist_path, "CFBundleName", display_name)?;
    set_string_key(plist_path, "CFBundleGetInfoString", display_name)
}

pub fn set_bundle_icon_file(plist_path: &Path, icon_file: &str) -> Result<()> {
    set_string_key(plist_path, "CFBundleIconFile", icon_file)
}

pub fn delete_key_if_exists(plist_path: &Path, key: &str) -> Result<()> {
    let plist = plist_path.display().to_string();
    let command = format!("Delete :{key}");
    let output = std::process::Command::new(PLIST_BUDDY)
        .args(["-c", &command, &plist])
        .output()
        .with_context(|| {
            format!(
                "Failed to launch PlistBuddy while deleting {key} from {}",
                plist_path.display()
            )
        })?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("Does Not Exist") {
        return Ok(());
    }

    Err(error::command_failed(
        PLIST_BUDDY,
        &["-c", &command, &plist],
        &output,
    ))
    .with_context(|| format!("Failed to delete {key} from {}", plist_path.display()))
}

pub fn set_localized_display_names(app_path: &Path, display_name: &str) -> Result<usize> {
    let resources_dir = app_path.join("Contents/Resources");
    if !resources_dir.exists() {
        return Ok(0);
    }

    let mut updated = 0usize;
    for entry in fs::read_dir(&resources_dir)
        .with_context(|| format!("Failed to read directory {}", resources_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let is_lproj = path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("lproj"));
        if !is_lproj {
            continue;
        }

        let strings_path = path.join("InfoPlist.strings");
        if !strings_path.exists() {
            continue;
        }

        set_string_key(&strings_path, "CFBundleDisplayName", display_name)?;
        set_string_key(&strings_path, "CFBundleName", display_name)?;
        updated += 1;
    }

    Ok(updated)
}

pub fn nested_bundle_plists(app_path: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut plists = Vec::new();
    collect_nested_bundle_plists(app_path, app_path, &mut plists)?;
    plists.sort();
    Ok(plists)
}

fn set_string_key(plist_path: &Path, key: &str, value: &str) -> Result<()> {
    let plist = plist_path.display().to_string();
    let command = format!("Set :{key} {}", plist_string_literal(value));

    error::run_command_with_privilege_fallback(PLIST_BUDDY, &["-c", &command, &plist])
        .with_context(|| format!("Failed to set {key} to {value} in {}", plist_path.display()))
}

fn plist_string_literal(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
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
