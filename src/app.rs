use crate::{
    error,
    icon::{self, IconSource},
    lang::Language,
    plist, prefs, sign,
};
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
    pub app_name: String,
}

#[derive(Debug, Clone)]
pub struct CreateOptions {
    pub index: u16,
    pub force: bool,
    pub app_name: Option<String>,
    pub icon: IconSource,
}

impl CreateOptions {
    pub fn default(index: u16) -> Self {
        Self {
            index,
            force: false,
            app_name: None,
            icon: IconSource::Default,
        }
    }

    pub fn target_path(&self) -> Result<PathBuf> {
        target_app_path(self.index, self.app_name.as_deref())
    }
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
        lines.push(format!("  [{}] {}", instance.index, instance.app_name));
        lines.push(format!("      {}", instance.path.display()));
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
    let mut options = CreateOptions::default(index);
    options.force = force;
    create_instance_with_progress(language, options, |_, _, _| {})
}

pub fn create_instance_with_progress<F>(
    language: Language,
    options: CreateOptions,
    mut report: F,
) -> Result<String>
where
    F: FnMut(usize, usize, &str),
{
    let index = options.index;
    ensure_valid_copy_index(index)?;
    let mut lines = Vec::new();

    let source = Path::new(ORIGINAL_APP_PATH);
    let target = options.target_path()?;

    if !source.exists() {
        bail!("{}", language.source_not_found());
    }

    if same_as_original(&target) {
        bail!("{}", language.refuse_overwrite_original());
    }

    let target_exists = target.exists();
    if target_exists {
        if !options.force {
            bail!("{}", language.target_exists(&target.display().to_string()));
        }
    }

    let source_str = source.display().to_string();
    let target_str = target.display().to_string();
    let total_steps = if target_exists && options.force {
        10
    } else {
        9
    };
    let mut current_step = 0usize;

    if target_exists && options.force {
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

    let display_name = display_name_for_target(index, options.app_name.as_deref());
    let update_name_message = language.updating_app_name(&display_name);
    current_step += 1;
    report(current_step, total_steps, &update_name_message);
    lines.push(update_name_message);
    plist::set_display_name(&target_plist, &display_name)?;
    let localized_name_updates = plist::set_localized_display_names(&target, &display_name)?;
    lines.push(language.app_name_updated(&display_name));
    lines.push(language.localized_app_names_updated(localized_name_updates));

    let icon_message = language.applying_icon(&options.icon.describe(language));
    current_step += 1;
    report(current_step, total_steps, &icon_message);
    lines.push(icon_message);
    icon::apply_icon(language, &target, index, &options.icon)?;
    if !matches!(options.icon, IconSource::Default) {
        plist::set_bundle_icon_file(&target_plist, icon::custom_icon_file_stem())?;
        plist::delete_key_if_exists(&target_plist, "CFBundleIconName")?;
    }
    lines.push(language.icon_applied().to_string());

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

    let Some(instance) = find_copy_by_index(index)? else {
        bail!(
            "{}",
            language.app_copy_not_found(&app_path(index).display().to_string())
        );
    };
    let target = instance.path;

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

    let Some(instance) = find_copy_by_index(index)? else {
        bail!(
            "{}",
            language.app_copy_not_found(&app_path(index).display().to_string())
        );
    };
    let target = instance.path;

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

    Ok(Some(build_instance(
        1,
        path,
        Some(BUNDLE_PREFIX.to_string()),
    )))
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

        if same_as_original(&path) {
            continue;
        }

        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };

        let bundle_id = plist::bundle_identifier(&plist_path(&path)).ok();
        let Some(index) = bundle_id
            .as_deref()
            .and_then(parse_copy_index_from_bundle_id)
            .or_else(|| parse_copy_name(name))
        else {
            continue;
        };

        copies.push(build_instance(index, path, bundle_id));
    }

    copies.sort_by(|left, right| {
        left.index
            .cmp(&right.index)
            .then_with(|| left.app_name.cmp(&right.app_name))
    });
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
    PathBuf::from(APPLICATIONS_DIR).join(default_copy_app_name(index))
}

pub fn default_copy_app_name(index: u16) -> String {
    format!("WeChat{index}.app")
}

pub fn target_app_path(index: u16, custom_name: Option<&str>) -> Result<PathBuf> {
    let app_name = match custom_name {
        Some(name) if !name.trim().is_empty() => normalize_app_name(name)?,
        _ => default_copy_app_name(index),
    };
    Ok(PathBuf::from(APPLICATIONS_DIR).join(app_name))
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

fn build_instance(index: u16, path: PathBuf, bundle_id: Option<String>) -> AppInstance {
    let app_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    AppInstance {
        index,
        path,
        bundle_id,
        app_name,
    }
}

fn find_copy_by_index(index: u16) -> Result<Option<AppInstance>> {
    Ok(scan_copies()?
        .into_iter()
        .find(|instance| instance.index == index))
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

fn parse_copy_index_from_bundle_id(bundle_id: &str) -> Option<u16> {
    let number = bundle_id.strip_prefix(BUNDLE_PREFIX)?;
    let index = number.parse::<u16>().ok()?;
    if index < 2 {
        return None;
    }
    Some(index)
}

fn normalize_app_name(name: &str) -> Result<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        bail!("Copy name cannot be empty.");
    }

    if trimmed.contains('/') || trimmed.contains('\0') {
        bail!("Copy name contains unsupported characters.");
    }

    let without_suffix = trimmed.strip_suffix(".app").unwrap_or(trimmed).trim();
    if without_suffix.is_empty() {
        bail!("Copy name cannot be empty.");
    }

    Ok(format!("{without_suffix}.app"))
}

fn display_name_for_target(index: u16, custom_name: Option<&str>) -> String {
    let app_name = custom_name
        .and_then(|name| normalize_app_name(name).ok())
        .unwrap_or_else(|| default_copy_app_name(index));
    app_name
        .strip_suffix(".app")
        .unwrap_or(app_name.as_str())
        .to_string()
}

fn same_as_original(path: &Path) -> bool {
    path == Path::new(ORIGINAL_APP_PATH)
}

fn nested_bundle_identifier(
    target_bundle_id: &str,
    original_nested_bundle_id: &str,
    plist_path: &Path,
) -> String {
    let original_root_bundle_id = BUNDLE_PREFIX;
    if let Some(suffix) =
        original_nested_bundle_id.strip_prefix(&format!("{original_root_bundle_id}."))
    {
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
