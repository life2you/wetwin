# wetwin

A lightweight macOS WeChat multi-instance manager written in Rust.

`wetwin` helps you create, launch, list, and remove local WeChat app copies by copying `WeChat.app`, changing the Bundle Identifier, clearing quarantine attributes, and re-signing the app locally.

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
wetwin list
wetwin create 2
wetwin create 2 --force
wetwin open 2
wetwin open all
wetwin remove 2
wetwin remove 2 --yes
wetwin doctor
```

## Commands

### `wetwin list`

- Checks whether `/Applications/WeChat.app` exists
- Scans `/Applications` for `WeChat2.app`, `WeChat3.app`, `WeChat4.app`, and so on
- Prints discovered app paths and Bundle Identifiers

### `wetwin create <index>`

Example:

```bash
wetwin create 2
```

What it does:

1. Verifies `/Applications/WeChat.app` exists
2. Creates `/Applications/WeChat{index}.app`
3. Updates `CFBundleIdentifier` to `com.tencent.xinWeChat{index}`
4. Clears quarantine attributes with `xattr -cr`
5. Applies local ad-hoc signing with `codesign --force --deep --sign -`

If the target already exists, use:

```bash
wetwin create 2 --force
```

### `wetwin open <target>`

Examples:

```bash
wetwin open 2
wetwin open all
```

- `wetwin open 2` launches `/Applications/WeChat2.app`
- `wetwin open all` launches `/Applications/WeChat.app` and all discovered copies

### `wetwin remove <index>`

Example:

```bash
wetwin remove 2
```

- Only indices `>= 2` are allowed
- The original `/Applications/WeChat.app` is never removed
- A confirmation prompt is shown unless `--yes` is passed

Skip confirmation:

```bash
wetwin remove 2 --yes
```

### `wetwin doctor`

Checks:

- Current platform is macOS
- `/Applications/WeChat.app` exists
- `/usr/libexec/PlistBuddy` exists
- `xattr`, `codesign`, and `open` are available
- `/Applications` is writable or may require `sudo`
- Existing copies contain `Info.plist`

## How It Works

`wetwin` only automates local app bundle operations. It does not inject into WeChat, change WeChat behavior, bypass login restrictions, manage accounts, or modify network traffic.

The first version uses these local operations:

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
