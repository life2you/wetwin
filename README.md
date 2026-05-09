# wetwin

A lightweight macOS WeChat multi-instance manager written in Rust.

`wetwin` helps you create, launch, inspect, and remove local WeChat app copies by copying `WeChat.app`, changing the Bundle Identifier, clearing quarantine attributes, and re-signing the app locally.

When an operation inside `/Applications` needs elevated privileges, `wetwin` first tries it normally and then falls back to a native macOS administrator permission prompt.

## Features

- Create multiple local WeChat instances
- Launch a specific instance
- Launch the original app and all discovered instances
- List existing instances and Bundle Identifiers
- Remove created instances safely
- Check the local macOS environment with `doctor`
- Request macOS administrator permission only when an operation needs it

## Platform

- macOS only
- Rust stable
- No app injection
- No feature patching
- No account or credential management

## Installation

### Homebrew

```bash
brew tap life2you/tap
brew install life2you/tap/wetwin
```

Maintainer release steps live in [RELEASING.md](/Users/life2you/vibeCodes/github/wetwin/RELEASING.md).

### Build from source

```bash
git clone https://github.com/life2you/wetwin.git
cd wetwin
cargo build --release
```

The binary will be available at:

```bash
./target/release/wetwin
```

### Install locally with Cargo

```bash
cargo install --path .
```

## Usage

```bash
wetwin
wetwin --lang zh
wetwin --lang en
```

## TUI Flow

Run `wetwin` to enter the terminal UI.

Inside the TUI you can:

- View the original WeChat app and all discovered copies
- Create the next suggested copy with a confirmation step and progress bar
- Open one copy or open all available apps
- Remove a selected copy with a confirmation step
- Run environment checks with `doctor`
- Switch UI language and persist it in config

The first launch prompts for language selection and stores it at:

```bash
~/Library/Application Support/wetwin/config.toml
```

## How It Works

`wetwin` only automates local app bundle operations. It does not inject into WeChat, change WeChat behavior, bypass login restrictions, manage accounts, or modify network traffic.

The current version uses these local operations:

```bash
cp -R /Applications/WeChat.app /Applications/WeChat2.app
/usr/libexec/PlistBuddy -c "Set :CFBundleIdentifier com.tencent.xinWeChat2" /Applications/WeChat2.app/Contents/Info.plist
xattr -cr /Applications/WeChat2.app
codesign --force --deep --sign - /Applications/WeChat2.app
open /Applications/WeChat2.app
rm -rf /Applications/WeChat2.app
```

## Risk Notes

- Creating, signing, or deleting apps inside `/Applications` may require `sudo`
- `wetwin` can trigger a native macOS administrator prompt through `osascript` when needed
- Local re-signing may need to be repeated after WeChat updates
- App bundle operations always carry some risk if the system or source app is already in an unusual state
- Please close WeChat copies before deleting them

## Disclaimer

`wetwin` is an app-copy management tool only. It does not provide login bypassing, account unlocking, reverse engineering, binary patching, or anti-detection capabilities. Users are responsible for complying with local law, platform rules, and WeChat terms.

## License

MIT
