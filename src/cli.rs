use crate::lang::Language;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "wetwin",
    version,
    about = "A lightweight macOS WeChat multi-instance manager written in Rust.",
    long_about = "wetwin launches a terminal UI for managing local WeChat app copies on macOS."
)]
pub struct Cli {
    #[arg(long, value_enum, global = true, help = "TUI language: en or zh.")]
    pub lang: Option<Language>,
}
