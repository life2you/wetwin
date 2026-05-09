use crate::lang::Language;
use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;

const DEFAULT_NEXT_COPY_INDEX: u16 = 2;

#[derive(Debug, Clone, Copy, Default)]
pub struct Config {
    pub language: Option<Language>,
    pub next_copy_index: Option<u16>,
}

pub fn load_config() -> Result<Option<Config>> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let text = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file {}", path.display()))?;

    let mut config = Config::default();

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("language=") {
            config.language = parse_language(value.trim());
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("next_copy_index=") {
            config.next_copy_index = parse_next_copy_index(value.trim());
        }
    }

    Ok(Some(config))
}

pub fn save_language(language: Language) -> Result<()> {
    let mut config = load_or_default()?;
    config.language = Some(language);
    save_config(config)
}

pub fn load_next_copy_index() -> Result<u16> {
    Ok(load_or_default()?
        .next_copy_index
        .unwrap_or(DEFAULT_NEXT_COPY_INDEX)
        .max(DEFAULT_NEXT_COPY_INDEX))
}

pub fn save_next_copy_index(index: u16) -> Result<()> {
    let mut config = load_or_default()?;
    config.next_copy_index = Some(index.max(DEFAULT_NEXT_COPY_INDEX));
    save_config(config)
}

fn load_or_default() -> Result<Config> {
    Ok(load_config()?.unwrap_or_default())
}

fn save_config(config: Config) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory {}", parent.display()))?;
    }

    let mut lines = Vec::new();
    if let Some(language) = config.language {
        lines.push(format!("language={}", language.code()));
    }
    if let Some(next_copy_index) = config.next_copy_index {
        lines.push(format!(
            "next_copy_index={}",
            next_copy_index.max(DEFAULT_NEXT_COPY_INDEX)
        ));
    }

    let mut text = lines.join("\n");
    if !text.is_empty() {
        text.push('\n');
    }

    fs::write(&path, text)
        .with_context(|| format!("Failed to write config file {}", path.display()))?;

    Ok(())
}

fn config_path() -> Result<PathBuf> {
    let home = env::var("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("wetwin")
        .join("config.toml"))
}

fn parse_language(value: &str) -> Option<Language> {
    match value.to_ascii_lowercase().as_str() {
        "zh" | "cn" | "中文" => Some(Language::Zh),
        "en" | "english" | "英文" => Some(Language::En),
        _ => None,
    }
}

fn parse_next_copy_index(value: &str) -> Option<u16> {
    let index = value.parse::<u16>().ok()?;
    if index < DEFAULT_NEXT_COPY_INDEX {
        return Some(DEFAULT_NEXT_COPY_INDEX);
    }
    Some(index)
}
