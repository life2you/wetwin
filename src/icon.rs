use crate::{error, lang::Language};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const ICONUTIL_PATH: &str = "/usr/bin/iconutil";
const SIPS_PATH: &str = "/usr/bin/sips";
const SWIFT_PATH: &str = "/usr/bin/swift";
const ICON_FILE_NAME: &str = "AppIcon.icns";
const CUSTOM_ICON_FILE_NAME: &str = "WetwinIcon.icns";
const LSREGISTER_PATH: &str = "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconPreset {
    Green,
    Blue,
    Orange,
    Red,
    Slate,
}

impl IconPreset {
    pub const ALL: [Self; 5] = [
        Self::Green,
        Self::Blue,
        Self::Orange,
        Self::Red,
        Self::Slate,
    ];

    pub fn all() -> &'static [Self] {
        &Self::ALL
    }

    pub fn label(self, language: Language) -> &'static str {
        match (self, language) {
            (Self::Green, Language::En) => "Green badge",
            (Self::Green, Language::Zh) => "绿色角标",
            (Self::Blue, Language::En) => "Blue badge",
            (Self::Blue, Language::Zh) => "蓝色角标",
            (Self::Orange, Language::En) => "Orange badge",
            (Self::Orange, Language::Zh) => "橙色角标",
            (Self::Red, Language::En) => "Red badge",
            (Self::Red, Language::Zh) => "红色角标",
            (Self::Slate, Language::En) => "Slate badge",
            (Self::Slate, Language::Zh) => "灰蓝角标",
        }
    }

    fn color_hex(self) -> &'static str {
        match self {
            Self::Green => "#22C55E",
            Self::Blue => "#2563EB",
            Self::Orange => "#F97316",
            Self::Red => "#DC2626",
            Self::Slate => "#475569",
        }
    }
}

#[derive(Debug, Clone)]
pub enum IconSource {
    Default,
    Preset(IconPreset),
}

impl IconSource {
    pub fn describe(&self, language: Language) -> String {
        match self {
            Self::Default => match language {
                Language::En => "Keep the original WeChat icon".to_string(),
                Language::Zh => "沿用原版微信图标".to_string(),
            },
            Self::Preset(preset) => match language {
                Language::En => format!("Preset icon: {}", preset.label(language)),
                Language::Zh => format!("预设图标：{}", preset.label(language)),
            },
        }
    }
}

pub fn iconutil_path() -> &'static str {
    ICONUTIL_PATH
}

pub fn sips_path() -> &'static str {
    SIPS_PATH
}

pub fn apply_icon(
    language: Language,
    app_path: &Path,
    copy_index: u16,
    source: &IconSource,
) -> Result<()> {
    match source {
        IconSource::Default => Ok(()),
        IconSource::Preset(preset) => apply_preset_icon(language, app_path, copy_index, *preset),
    }
}

fn apply_preset_icon(
    language: Language,
    app_path: &Path,
    copy_index: u16,
    preset: IconPreset,
) -> Result<()> {
    let temp_dir = temp_work_dir("preset-icon")?;
    let base_icon = icon_file_path(app_path);
    let rendered_png = temp_dir.join("preset.png");
    let rendered_icns = temp_dir.join(ICON_FILE_NAME);
    let swift_script = temp_dir.join("render_preset.swift");

    fs::write(&swift_script, render_preset_script())
        .with_context(|| format!("Failed to write {}", swift_script.display()))?;

    let base_icon_str = base_icon.display().to_string();
    let rendered_png_str = rendered_png.display().to_string();
    let swift_script_str = swift_script.display().to_string();
    let badge_text = copy_index.to_string();
    let color_hex = preset.color_hex();

    error::run_command(
        SWIFT_PATH,
        &[
            &swift_script_str,
            &base_icon_str,
            &rendered_png_str,
            &badge_text,
            color_hex,
        ],
    )
    .with_context(|| language.icon_preset_render_failed(preset.label(language)))?;

    build_icns_from_image(language, &rendered_png, &rendered_icns)?;
    install_icon(language, app_path, &rendered_icns)
}

fn build_icns_from_image(
    language: Language,
    source_image: &Path,
    output_icns: &Path,
) -> Result<()> {
    let iconset_dir = output_icns.with_extension("iconset");
    fs::create_dir_all(&iconset_dir)
        .with_context(|| format!("Failed to create {}", iconset_dir.display()))?;

    let source_str = source_image.display().to_string();

    for &(size, scale) in iconset_specifications() {
        let pixel_size = size * scale;
        let file_name = if scale == 1 {
            format!("icon_{size}x{size}.png")
        } else {
            format!("icon_{size}x{size}@2x.png")
        };
        let output_path = iconset_dir.join(file_name);
        let output_path_str = output_path.display().to_string();
        let pixel_size_str = pixel_size.to_string();

        error::run_command(
            SIPS_PATH,
            &[
                "-s",
                "format",
                "png",
                "--resampleHeightWidth",
                &pixel_size_str,
                &pixel_size_str,
                &source_str,
                "--out",
                &output_path_str,
            ],
        )
        .with_context(|| language.icon_generate_failed(&source_image.display().to_string()))?;
    }

    let iconset_dir_str = iconset_dir.display().to_string();
    let output_icns_str = output_icns.display().to_string();
    error::run_command(
        ICONUTIL_PATH,
        &["-c", "icns", &iconset_dir_str, "-o", &output_icns_str],
    )
    .with_context(|| language.icon_pack_failed(&output_icns.display().to_string()))?;

    Ok(())
}

fn install_icon(language: Language, app_path: &Path, generated_icon: &Path) -> Result<()> {
    let target = custom_icon_file_path(app_path);
    let source_str = generated_icon.display().to_string();
    let target_str = target.display().to_string();
    error::run_command_with_privilege_fallback("cp", &[&source_str, &target_str])
        .with_context(|| language.icon_install_failed(&target.display().to_string()))?;

    let legacy_target = icon_file_path(app_path);
    let legacy_target_str = legacy_target.display().to_string();
    error::run_command_with_privilege_fallback("cp", &[&source_str, &legacy_target_str])
        .with_context(|| language.icon_install_failed(&legacy_target.display().to_string()))?;

    refresh_bundle_icon(app_path, &target)
}

fn icon_file_path(app_path: &Path) -> PathBuf {
    app_path.join("Contents/Resources").join(ICON_FILE_NAME)
}

pub fn custom_icon_file_stem() -> &'static str {
    "WetwinIcon"
}

fn custom_icon_file_path(app_path: &Path) -> PathBuf {
    app_path
        .join("Contents/Resources")
        .join(CUSTOM_ICON_FILE_NAME)
}

fn temp_work_dir(label: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("System clock is before the Unix epoch")?
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("wetwin-{label}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).with_context(|| format!("Failed to create {}", dir.display()))?;
    Ok(dir)
}

fn iconset_specifications() -> &'static [(u32, u32)] {
    &[
        (16, 1),
        (16, 2),
        (32, 1),
        (32, 2),
        (128, 1),
        (128, 2),
        (256, 1),
        (256, 2),
        (512, 1),
        (512, 2),
    ]
}

fn render_preset_script() -> &'static str {
    r##"import AppKit

func color(from hex: String) -> NSColor {
    var cleaned = hex.trimmingCharacters(in: .whitespacesAndNewlines)
    if cleaned.hasPrefix("#") {
        cleaned.removeFirst()
    }

    guard cleaned.count == 6, let value = Int(cleaned, radix: 16) else {
        return NSColor.systemBlue
    }

    let red = CGFloat((value >> 16) & 0xFF) / 255.0
    let green = CGFloat((value >> 8) & 0xFF) / 255.0
    let blue = CGFloat(value & 0xFF) / 255.0
    return NSColor(calibratedRed: red, green: green, blue: blue, alpha: 1.0)
}

let args = CommandLine.arguments
guard args.count >= 5 else {
    fputs("Usage: render_preset.swift <base-icon> <output-png> <badge-text> <hex-color>\n", stderr)
    exit(1)
}

let baseIconPath = args[1]
let outputPath = args[2]
let badgeText = args[3]
let badgeColor = color(from: args[4])

guard let baseImage = NSImage(contentsOfFile: baseIconPath) else {
    fputs("Failed to load base icon at \(baseIconPath)\n", stderr)
    exit(1)
}

let size = NSSize(width: 1024, height: 1024)
let canvas = NSImage(size: size)
canvas.lockFocus()
NSGraphicsContext.current?.imageInterpolation = .high
baseImage.draw(
    in: NSRect(origin: .zero, size: size),
    from: NSRect(origin: .zero, size: baseImage.size),
    operation: .sourceOver,
    fraction: 1.0
)

let badgeRect = NSRect(x: 700, y: 700, width: 280, height: 280)
let badgePath = NSBezierPath(roundedRect: badgeRect, xRadius: 140, yRadius: 140)
badgeColor.setFill()
badgePath.fill()

NSColor.white.withAlphaComponent(0.92).setStroke()
badgePath.lineWidth = 24
badgePath.stroke()

let paragraph = NSMutableParagraphStyle()
paragraph.alignment = .center
let font = NSFont.systemFont(ofSize: 148, weight: .bold)
let attributes: [NSAttributedString.Key: Any] = [
    .font: font,
    .foregroundColor: NSColor.white,
    .paragraphStyle: paragraph
]

let attributedText = NSAttributedString(string: badgeText, attributes: attributes)
let textSize = attributedText.size()
let textRect = NSRect(
    x: badgeRect.midX - textSize.width / 2,
    y: badgeRect.midY - textSize.height / 2 - 10,
    width: textSize.width,
    height: textSize.height
)
attributedText.draw(in: textRect)

canvas.unlockFocus()

guard
    let tiffData = canvas.tiffRepresentation,
    let bitmap = NSBitmapImageRep(data: tiffData),
    let pngData = bitmap.representation(using: .png, properties: [:])
else {
    fputs("Failed to render preset icon output.\n", stderr)
    exit(1)
}

if !FileManager.default.createFile(atPath: outputPath, contents: pngData) {
    fputs("Failed to write rendered icon to \(outputPath)\n", stderr)
    exit(1)
}
"##
}

fn refresh_bundle_icon(app_path: &Path, icon_path: &Path) -> Result<()> {
    let script = temp_work_dir("icon-refresh")?.join("refresh_icon.swift");
    fs::write(&script, refresh_icon_script())
        .with_context(|| format!("Failed to write {}", script.display()))?;

    let script_str = script.display().to_string();
    let icon_str = icon_path.display().to_string();
    let app_str = app_path.display().to_string();

    error::run_command(SWIFT_PATH, &[&script_str, &icon_str, &app_str])
        .with_context(|| format!("Failed to refresh icon for {}", app_path.display()))?;

    error::run_command_with_privilege_fallback("touch", &[&app_str]).with_context(|| {
        format!(
            "Failed to refresh bundle timestamp for {}",
            app_path.display()
        )
    })?;

    if Path::new(LSREGISTER_PATH).exists() {
        let _ = error::run_command(LSREGISTER_PATH, &["-f", &app_str]);
    }

    Ok(())
}

fn refresh_icon_script() -> &'static str {
    r##"import AppKit

let args = CommandLine.arguments
guard args.count >= 3 else {
    fputs("Usage: refresh_icon.swift <icon-path> <app-path>\n", stderr)
    exit(1)
}

let iconPath = args[1]
let appPath = args[2]

guard let image = NSImage(contentsOfFile: iconPath) else {
    fputs("Failed to load icon image at \(iconPath)\n", stderr)
    exit(1)
}

let ok = NSWorkspace.shared.setIcon(image, forFile: appPath, options: [])
if !ok {
    fputs("NSWorkspace failed to assign the custom icon.\n", stderr)
    exit(1)
}
"##
}
