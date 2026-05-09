use anyhow::{anyhow, Context, Result};
use std::process::{Command, Output};

pub fn command_failed(program: &str, args: &[&str], output: &Output) -> anyhow::Error {
    let mut details = String::new();

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !stdout.is_empty() {
        details.push_str(&format!("stdout: {stdout}"));
    }

    if !stderr.is_empty() {
        if !details.is_empty() {
            details.push(' ');
        }
        details.push_str(&format!("stderr: {stderr}"));
    }

    let permission_hint = if contains_permission_denied(&details) {
        " Permission denied. You may need to run this command with sudo."
    } else {
        ""
    };

    let suffix = if details.is_empty() {
        String::new()
    } else {
        format!(" {details}")
    };

    anyhow!(
        "Command failed: {} {}.{}{}",
        program,
        args.join(" "),
        suffix,
        permission_hint
    )
}

pub fn run_command(program: &str, args: &[&str]) -> Result<()> {
    let output = run_command_output(program, args)?;

    if output.status.success() {
        Ok(())
    } else {
        Err(command_failed(program, args, &output))
    }
}

pub fn run_command_with_privilege_fallback(program: &str, args: &[&str]) -> Result<()> {
    let output = run_command_output(program, args)?;

    if output.status.success() {
        return Ok(());
    }

    let details = output_details(&output);
    if !contains_permission_denied(&details) {
        return Err(command_failed(program, args, &output));
    }

    run_command_as_administrator(program, args)
}

pub fn run_command_capture(program: &str, args: &[&str]) -> Result<String> {
    let output = run_command_output(program, args)?;

    if !output.status.success() {
        return Err(command_failed(program, args, &output));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn run_command_output(program: &str, args: &[&str]) -> Result<Output> {
    Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("Failed to launch command: {program} {}", args.join(" ")))
}

fn run_command_as_administrator(program: &str, args: &[&str]) -> Result<()> {
    let shell_command = shell_command(program, args);
    let script = format!(
        "do shell script {} with administrator privileges",
        apple_script_string_literal(&shell_command)
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .with_context(|| "Failed to launch osascript for administrator privileges".to_string())?;

    if output.status.success() {
        return Ok(());
    }

    let details = output_details(&output);
    if details.to_ascii_lowercase().contains("user canceled") {
        return Err(anyhow!("Administrator permission request was cancelled."));
    }

    Err(anyhow!(
        "Administrator permission request failed while running: {} {}. {}",
        program,
        args.join(" "),
        details
    ))
}

fn shell_command(program: &str, args: &[&str]) -> String {
    let mut parts = Vec::with_capacity(args.len() + 1);
    parts.push(shell_quote(program));
    for arg in args {
        parts.push(shell_quote(arg));
    }
    parts.join(" ")
}

fn shell_quote(value: &str) -> String {
    let escaped = value.replace('\'', "'\"'\"'");
    format!("'{escaped}'")
}

fn apple_script_string_literal(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn output_details(output: &Output) -> String {
    let mut details = String::new();

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !stdout.is_empty() {
        details.push_str(&format!("stdout: {stdout}"));
    }

    if !stderr.is_empty() {
        if !details.is_empty() {
            details.push(' ');
        }
        details.push_str(&format!("stderr: {stderr}"));
    }

    details
}

fn contains_permission_denied(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("permission denied")
        || lower.contains("operation not permitted")
        || lower.contains("not permitted")
}
