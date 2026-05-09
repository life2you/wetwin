use crate::lang::Language;
use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub language: Language,
}

pub fn load_config() -> Result<Option<Config>> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let text = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file {}", path.display()))?;

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("language=") {
            if let Some(language) = parse_language(value.trim()) {
                return Ok(Some(Config { language }));
            }
        }
    }

    Ok(None)
}

pub fn save_language(language: Language) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory {}", parent.display()))?;
    }

    fs::write(&path, format!("language={}\n", language.code()))
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
