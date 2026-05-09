use crate::{error, lang::Language};
use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;

const DEFAULTS: &str = "defaults";
const ORIGINAL_BUNDLE_ID: &str = "com.tencent.xinWeChat";
const FALLBACK_LANGUAGE: &str = "zh-Hans";
const FALLBACK_LOCALE: &str = "zh_Hans_CN";

pub enum CopyPrefsOutcome {
    Copied,
    MissingSource,
}

pub fn copy_original_preferences(
    language: Language,
    target_bundle_id: &str,
) -> Result<CopyPrefsOutcome> {
    if target_bundle_id == ORIGINAL_BUNDLE_ID {
        return Ok(CopyPrefsOutcome::MissingSource);
    }

    let exported = match error::run_command_capture(DEFAULTS, &["export", ORIGINAL_BUNDLE_ID, "-"])
    {
        Ok(value) => value,
        Err(err) => {
            let text = err.to_string();
            if text.contains("Domain") && text.contains("does not exist") {
                return Ok(CopyPrefsOutcome::MissingSource);
            }
            return Err(err).with_context(|| language.preferences_export_failed());
        }
    };

    let temp_path = export_temp_path(target_bundle_id)?;
    fs::write(&temp_path, exported.as_bytes()).with_context(|| {
        format!(
            "Failed to write exported preferences to {}",
            temp_path.display()
        )
    })?;

    let temp_str = temp_path.display().to_string();
    let import_result = error::run_command(DEFAULTS, &["import", target_bundle_id, &temp_str])
        .with_context(|| language.preferences_import_failed(target_bundle_id));

    let _ = fs::remove_file(&temp_path);

    import_result?;
    Ok(CopyPrefsOutcome::Copied)
}

pub fn apply_explicit_language(language: Language, target_bundle_id: &str) -> Result<()> {
    let preferred_language = preferred_language();
    let preferred_locale = preferred_locale();

    error::run_command(
        DEFAULTS,
        &[
            "write",
            target_bundle_id,
            "AppleLanguages",
            "-array",
            &preferred_language,
        ],
    )
    .with_context(|| language.language_write_failed(target_bundle_id))?;

    error::run_command(
        DEFAULTS,
        &[
            "write",
            target_bundle_id,
            "AppleLocale",
            "-string",
            &preferred_locale,
        ],
    )
    .with_context(|| language.locale_write_failed(target_bundle_id))?;

    Ok(())
}

pub fn preferred_language() -> String {
    original_apple_language().unwrap_or_else(|| FALLBACK_LANGUAGE.to_string())
}

pub fn preferred_locale() -> String {
    original_apple_locale().unwrap_or_else(|| FALLBACK_LOCALE.to_string())
}

fn export_temp_path(target_bundle_id: &str) -> Result<PathBuf> {
    let mut path = env::temp_dir();
    path.push(format!(
        "wetwin-{}-preferences.plist",
        target_bundle_id.replace('.', "-")
    ));
    Ok(path)
}

fn original_apple_language() -> Option<String> {
    read_defaults_value(ORIGINAL_BUNDLE_ID, "AppleLanguages")
        .and_then(|raw| parse_defaults_array_first(&raw))
        .or_else(|| {
            read_defaults_value("-g", "AppleLanguages")
                .and_then(|raw| parse_defaults_array_first(&raw))
        })
}

fn original_apple_locale() -> Option<String> {
    read_defaults_value(ORIGINAL_BUNDLE_ID, "AppleLocale")
        .or_else(|| read_defaults_value("-g", "AppleLocale"))
}

fn read_defaults_value(domain: &str, key: &str) -> Option<String> {
    error::run_command_capture(DEFAULTS, &["read", domain, key]).ok()
}

fn parse_defaults_array_first(raw: &str) -> Option<String> {
    raw.lines()
        .map(str::trim)
        .find(|line| {
            !line.is_empty()
                && *line != "("
                && *line != ")"
                && *line != ","
                && !line.starts_with(')')
        })
        .map(|line| line.trim_end_matches(',').trim_matches('"').to_string())
}
