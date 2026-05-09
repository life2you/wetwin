use crate::{error, lang::Language, plist, prefs, sign};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub const APPLICATIONS_DIR: &str = "/Applications";
pub const ORIGINAL_APP_NAME: &str = "WeChat.app";
pub const ORIGINAL_APP_PATH: &str = "/Applications/WeChat.app";
const BUNDLE_PREFIX: &str = "com.tencent.xinWeChat";

#[derive(Debug, Clone)]
pub struct AppInstance {
    pub index: u16,
    pub path: PathBuf,
    pub bundle_id: Option<String>,
}

pub fn ensure_supported_platform(language: Language) -> Result<()> {
    if cfg!(target_os = "macos") {
        Ok(())
    } else {
        bail!("{}", language.only_supports_macos())
    }
}

pub fn list_instances(language: Language) -> Result<String> {
    let original = original_instance()?;
    let copies = scan_copies()?;
    let mut lines = Vec::new();

    lines.push(language.wechat_installation().to_string());
    lines.push(language.original_label().to_string());

    match original {
        Some(instance) => {
            lines.push(format!(
                "  {} {}",
                language.found(),
                instance.path.display()
            ));
            lines.push(format!(
                "  {}: {}",
                language.bundle_id_label(),
                instance
                    .bundle_id
                    .unwrap_or_else(|| language.unavailable().to_string())
            ));
        }
        None => {
            lines.push(format!("  {} {}", language.missing(), ORIGINAL_APP_PATH));
        }
    }

    lines.push(String::new());
    lines.push(language.local_copies().to_string());

    if copies.is_empty() {
        lines.push(format!("  {}", language.no_local_copies()));
        return Ok(lines.join("\n"));
    }

    for instance in copies {
        lines.push(format!(
            "  [{}] {}",
            instance.index,
            instance.path.display()
        ));
        lines.push(format!(
            "      {}: {}",
            language.bundle_id_label(),
            instance
                .bundle_id
                .unwrap_or_else(|| language.unavailable().to_string())
        ));
    }

    Ok(lines.join("\n"))
}

pub fn create_instance(language: Language, index: u16, force: bool) -> Result<String> {
    ensure_valid_copy_index(index)?;
    let mut lines = Vec::new();

    let source = Path::new(ORIGINAL_APP_PATH);
    let target = app_path(index);

    if !source.exists() {
        bail!("{}", language.source_not_found());
    }

    if target.exists() {
        if !force {
            bail!("{}", language.target_exists(&target.display().to_string()));
        }

        lines.push(format!(
            "{} {}",
            language.removing_existing_copy(),
            target.display()
        ));
        remove_app_at_path(language, &target)?;
    }

    let source_str = source.display().to_string();
    let target_str = target.display().to_string();

    lines.push(format!(
        "{} {} -> {}",
        language.copying(),
        source.display(),
        target.display()
    ));
    error::run_command_with_privilege_fallback("cp", &["-R", &source_str, &target_str])
        .with_context(|| {
            format!(
                "Failed to copy {} to {}",
                source.display(),
                target.display()
            )
        })?;

    let target_plist = plist_path(&target);
    if !target_plist.exists() {
        bail!(
            "{}",
            language.info_plist_missing(&target_plist.display().to_string())
        );
    }

    let bundle_id = bundle_id_for_index(index);
    lines.push(language.updating_bundle_id(&bundle_id));
    plist::set_bundle_identifier(&target_plist, &bundle_id)?;

    lines.push(language.copying_preferences().to_string());
    match prefs::copy_original_preferences(language, &bundle_id) {
        Ok(prefs::CopyPrefsOutcome::Copied) => {
            lines.push(language.preferences_copied().to_string());
        }
        Ok(prefs::CopyPrefsOutcome::MissingSource) => {
            lines.push(language.preferences_not_found().to_string());
        }
        Err(err) => {
            lines.push(language.preferences_copy_warning(&err.to_string()));
        }
    }

    lines.push(language.applying_language_preference().to_string());
    prefs::apply_explicit_language(language, &bundle_id)?;
    lines.push(language.language_preference_applied().to_string());

    lines.push(language.clearing_quarantine().to_string());
    sign::clear_quarantine(&target)?;

    lines.push(language.signing().to_string());
    sign::ad_hoc_sign(&target)?;

    lines.push(format!(
        "{} {} ({bundle_id})",
        language.created_successfully(),
        target.display()
    ));

    Ok(lines.join("\n"))
}

pub fn open_instance(language: Language, index: u16) -> Result<String> {
    ensure_valid_copy_index(index)?;

    let target = app_path(index);
    if !target.exists() {
        bail!(
            "{}",
            language.app_copy_not_found(&target.display().to_string())
        );
    }

    open_app(&target)?;
    Ok(format!("{} {}", language.opened(), target.display()))
}

pub fn open_all(language: Language) -> Result<String> {
    let mut targets = Vec::new();
    let mut lines = Vec::new();

    let original = PathBuf::from(ORIGINAL_APP_PATH);
    if original.exists() {
        targets.push(original);
    } else {
        lines.push(format!(
            "{} {}",
            language.original_warning_not_found(),
            ORIGINAL_APP_PATH
        ));
    }

    for copy in scan_copies()? {
        targets.push(copy.path);
    }

    if targets.is_empty() {
        bail!("{}", language.no_wechat_apps_to_open());
    }

    for app in &targets {
        open_app(app)?;
    }

    lines.push(language.opened_count(targets.len()));
    Ok(lines.join("\n"))
}

pub fn remove_instance(language: Language, index: u16, confirmed: bool) -> Result<String> {
    ensure_valid_copy_index(index)?;

    let target = app_path(index);
    if !target.exists() {
        bail!(
            "{}",
            language.app_copy_not_found(&target.display().to_string())
        );
    }

    if same_as_original(&target) {
        bail!("{}", language.refuse_remove_original());
    }

    if !confirmed {
        bail!("{}", language.removal_requires_confirmation());
    }

    remove_app_at_path(language, &target)?;
    Ok(format!("{} {}", language.removed(), target.display()))
}

pub fn original_instance() -> Result<Option<AppInstance>> {
    let path = PathBuf::from(ORIGINAL_APP_PATH);
    if !path.exists() {
        return Ok(None);
    }

    Ok(Some(build_instance(1, path)))
}

pub fn scan_copies() -> Result<Vec<AppInstance>> {
    let mut copies = Vec::new();

    for entry in fs::read_dir(APPLICATIONS_DIR)
        .with_context(|| format!("Failed to read {APPLICATIONS_DIR}"))?
    {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };

        let Some(index) = parse_copy_name(name) else {
            continue;
        };

        copies.push(build_instance(index, path));
    }

    copies.sort_by_key(|instance| instance.index);
    Ok(copies)
}

pub fn suggested_available_indices(limit: usize) -> Result<Vec<u16>> {
    let existing = scan_copies()?;
    let mut next_index = 2u16;
    let mut result = Vec::new();

    while result.len() < limit {
        if existing.iter().all(|instance| instance.index != next_index) {
            result.push(next_index);
        }
        next_index = next_index.saturating_add(1);
        if next_index == u16::MAX {
            break;
        }
    }

    Ok(result)
}

pub fn app_path(index: u16) -> PathBuf {
    PathBuf::from(format!("{APPLICATIONS_DIR}/WeChat{index}.app"))
}

pub fn plist_path(app_path: &Path) -> PathBuf {
    app_path.join("Contents/Info.plist")
}

pub fn bundle_id_for_index(index: u16) -> String {
    format!("{BUNDLE_PREFIX}{index}")
}

fn open_app(path: &Path) -> Result<()> {
    let app = path.display().to_string();
    if path == Path::new(ORIGINAL_APP_PATH) {
        return error::run_command("open", &[&app])
            .with_context(|| format!("Failed to open {}", path.display()));
    }

    let preferred_language = prefs::preferred_language();
    let preferred_locale = prefs::preferred_locale();
    let apple_languages_arg = format!("({preferred_language})");

    error::run_command(
        "open",
        &[
            &app,
            "--args",
            "-AppleLanguages",
            &apple_languages_arg,
            "-AppleLocale",
            &preferred_locale,
        ],
    )
    .with_context(|| format!("Failed to open {}", path.display()))
}

fn remove_app_at_path(language: Language, path: &Path) -> Result<()> {
    if same_as_original(path) {
        bail!("{}", language.refuse_remove_original());
    }

    let target = path.display().to_string();
    error::run_command_with_privilege_fallback("rm", &["-rf", &target])
        .with_context(|| format!("Failed to remove {}", path.display()))
}

fn build_instance(index: u16, path: PathBuf) -> AppInstance {
    let bundle_id = plist::bundle_identifier(&plist_path(&path)).ok();
    AppInstance {
        index,
        path,
        bundle_id,
    }
}

fn ensure_valid_copy_index(index: u16) -> Result<()> {
    if index < 2 {
        bail!("Index must be 2 or greater.");
    }
    Ok(())
}

fn parse_copy_name(name: &str) -> Option<u16> {
    if name == ORIGINAL_APP_NAME {
        return None;
    }

    let number = name.strip_prefix("WeChat")?.strip_suffix(".app")?;
    let index = number.parse::<u16>().ok()?;
    if index < 2 {
        return None;
    }

    Some(index)
}

fn same_as_original(path: &Path) -> bool {
    path == Path::new(ORIGINAL_APP_PATH)
}
