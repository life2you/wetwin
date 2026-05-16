use crate::{app, icon, lang::Language, plist};
use anyhow::Result;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

pub fn run(language: Language) -> Result<String> {
    let mut lines = Vec::new();
    lines.push(language.doctor_title().to_string());
    lines.push(language.doctor_intro().to_string());
    lines.push(String::new());

    print_check(
        &mut lines,
        language,
        language.doctor_running_on_macos(),
        cfg!(target_os = "macos"),
        if cfg!(target_os = "macos") {
            None
        } else {
            Some(language.only_supports_macos())
        },
    );

    print_check(
        &mut lines,
        language,
        language.doctor_original_exists(),
        Path::new(app::ORIGINAL_APP_PATH).exists(),
        if Path::new(app::ORIGINAL_APP_PATH).exists() {
            None
        } else {
            Some(language.doctor_expected_source())
        },
    );

    print_check(
        &mut lines,
        language,
        language.doctor_plistbuddy_exists(),
        Path::new(plist::plist_buddy_path()).exists(),
        if Path::new(plist::plist_buddy_path()).exists() {
            None
        } else {
            Some(language.doctor_plistbuddy_note())
        },
    );

    print_command_exists(&mut lines, language, "xattr", language.doctor_xattr_note());
    print_command_exists(
        &mut lines,
        language,
        "codesign",
        language.doctor_codesign_note(),
    );
    print_command_exists(&mut lines, language, "open", language.doctor_open_note());
    print_command_exists(
        &mut lines,
        language,
        "osascript",
        language.doctor_osascript_note(),
    );
    print_path_exists(
        &mut lines,
        language,
        icon::sips_path(),
        language.doctor_sips_note(),
    );
    print_path_exists(
        &mut lines,
        language,
        icon::iconutil_path(),
        language.doctor_iconutil_note(),
    );

    let can_write_applications = command_success("test", &["-w", app::APPLICATIONS_DIR]);
    print_check(
        &mut lines,
        language,
        language.doctor_can_write_applications(),
        can_write_applications,
        Some(language.doctor_admin_prompt_note()),
    );

    let copies = app::scan_copies()?;
    if copies.is_empty() {
        lines.push(String::new());
        lines.push(language.doctor_no_local_copies().to_string());
        return Ok(lines.join("\n"));
    }

    lines.push(String::new());
    lines.push(language.doctor_existing_copies().to_string());
    for instance in copies {
        let plist_path = app::plist_path(&instance.path);
        let ok = plist_path.exists();
        let detail = format!(
            "{} | Info.plist: {}",
            instance.path.display(),
            plist_path.display()
        );
        print_check(
            &mut lines,
            language,
            &language.doctor_copy_label(instance.index),
            ok,
            Some(&detail),
        );
    }

    Ok(lines.join("\n"))
}

fn print_command_exists(lines: &mut Vec<String>, language: Language, command: &str, note: &str) {
    let ok = command_success("which", &[command]);
    print_check(
        lines,
        language,
        &language.doctor_command_available(command),
        ok,
        Some(note),
    );
}

fn print_path_exists(lines: &mut Vec<String>, language: Language, path: &str, note: &str) {
    print_check(
        lines,
        language,
        &language.doctor_command_available(path),
        Path::new(path).exists(),
        Some(note),
    );
}

fn command_success(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn print_check(
    lines: &mut Vec<String>,
    language: Language,
    name: &str,
    ok: bool,
    note: Option<&str>,
) {
    let status = if ok { language.ok() } else { language.warn() };
    lines.push(format!("[{}] {}", status, name));
    if let Some(note) = note {
        lines.push(format!("      {note}"));
    }
}
