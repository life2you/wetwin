mod app;
mod cli;
mod config;
mod doctor;
mod error;
mod lang;
mod plist;
mod prefs;
mod sign;
mod tui;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use lang::Language;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let saved_language = config::load_config()?.and_then(|config| config.language);
    let language = cli.lang.or(saved_language).unwrap_or(Language::Zh);
    let should_prompt_for_language = cli.lang.is_none() && saved_language.is_none();

    app::ensure_supported_platform(language)?;
    tui::run(language, should_prompt_for_language)
}
