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

#[allow(dead_code)]
pub fn create_instance(language: Language, index: u16, force: bool) -> Result<String> {
    create_instance_with_progress(language, index, force, |_, _, _| {})
}

pub fn create_instance_with_progress<F>(
    language: Language,
    index: u16,
    force: bool,
    mut report: F,
) -> Result<String>
where
    F: FnMut(usize, usize, &str),
{
    ensure_valid_copy_index(index)?;
    let mut lines = Vec::new();

    let source = Path::new(ORIGINAL_APP_PATH);
    let target = app_path(index);

    if !source.exists() {
        bail!("{}", language.source_not_found());
    }

    let target_exists = target.exists();
    if target_exists {
        if !force {
            bail!("{}", language.target_exists(&target.display().to_string()));
        }
    }

    let source_str = source.display().to_string();
    let target_str = target.display().to_string();
    let total_steps = if target_exists && force { 8 } else { 7 };
    let mut current_step = 0usize;

    if target_exists && force {
        let removal_message = format!("{} {}", language.removing_existing_copy(), target.display());
        current_step += 1;
        report(current_step, total_steps, &removal_message);
        lines.push(removal_message);
        remove_app_at_path(language, &target)?;
    }

    let copy_message = format!(
        "{} {} -> {}",
        language.copying(),
        source.display(),
        target.display()
    );
    current_step += 1;
    report(current_step, total_steps, &copy_message);
    lines.push(copy_message);
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
    let update_bundle_id_message = language.updating_bundle_id(&bundle_id);
    current_step += 1;
    report(current_step, total_steps, &update_bundle_id_message);
    lines.push(update_bundle_id_message);
    plist::set_bundle_identifier(&target_plist, &bundle_id)?;

    let nested_bundle_update_message = language.updating_nested_bundle_ids().to_string();
    current_step += 1;
    report(current_step, total_steps, &nested_bundle_update_message);
    lines.push(nested_bundle_update_message);
    let nested_plists = plist::nested_bundle_plists(&target)?;
    for nested_plist in &nested_plists {
        let original_nested_bundle_id = plist::bundle_identifier(nested_plist)?;
        let nested_bundle_id =
            nested_bundle_identifier(&bundle_id, &original_nested_bundle_id, nested_plist);
        plist::set_bundle_identifier(nested_plist, &nested_bundle_id)?;
    }
    lines.push(language.nested_bundle_ids_updated(nested_plists.len()));

    let copy_prefs_message = language.copying_preferences().to_string();
    current_step += 1;
    report(current_step, total_steps, &copy_prefs_message);
    lines.push(copy_prefs_message);
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

    let apply_language_message = language.applying_language_preference().to_string();
    current_step += 1;
    report(current_step, total_steps, &apply_language_message);
    lines.push(apply_language_message);
    prefs::apply_explicit_language(language, &bundle_id)?;
    lines.push(language.language_preference_applied().to_string());

    let clear_quarantine_message = language.clearing_quarantine().to_string();
    current_step += 1;
    report(current_step, total_steps, &clear_quarantine_message);
    lines.push(clear_quarantine_message);
    sign::clear_quarantine(&target)?;

    let sign_message = language.signing().to_string();
    current_step += 1;
    report(current_step, total_steps, &sign_message);
    lines.push(sign_message);
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

pub fn suggested_available_indices_from(start_index: u16, count: usize) -> Result<Vec<u16>> {
    let existing = scan_copies()?;
    let mut next_index = start_index.max(2);
    let mut result = Vec::new();

    while result.len() < count {
        if existing.iter().all(|instance| instance.index != next_index) {
            result.push(next_index);
        }

        if next_index == u16::MAX {
            break;
        }

        next_index = next_index.saturating_add(1);
    }

    Ok(result)
}

pub fn next_available_index_from(start_index: u16) -> Result<u16> {
    suggested_available_indices_from(start_index, 1)?
        .into_iter()
        .next()
        .context("No available copy index could be determined.")
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

fn nested_bundle_identifier(target_bundle_id: &str, original_nested_bundle_id: &str, plist_path: &Path) -> String {
    let original_root_bundle_id = BUNDLE_PREFIX;
    if let Some(suffix) = original_nested_bundle_id.strip_prefix(&format!("{original_root_bundle_id}.")) {
        return format!("{target_bundle_id}.{suffix}");
    }

    if let Some(last_segment) = original_nested_bundle_id.rsplit('.').next() {
        if !last_segment.is_empty() {
            return format!("{target_bundle_id}.{last_segment}");
        }
    }

    let bundle_name = plist_path
        .parent()
        .and_then(|contents| contents.parent())
        .and_then(|bundle_dir| bundle_dir.file_stem())
        .and_then(|value| value.to_str())
        .unwrap_or("NestedBundle");
    format!("{target_bundle_id}.{bundle_name}")
}
