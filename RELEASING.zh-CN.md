[English](RELEASING.md) | [简体中文](RELEASING.zh-CN.md)

# 发布 `wetwin`

这份文档是发布 `wetwin` 新版本、上传预编译 macOS 二进制并更新 Homebrew 的维护者 SOP。

## 前置条件

- 工作区是干净的
- `cargo check` 通过
- `cargo test` 通过
- `Cargo.toml` 和 `Cargo.lock` 已经是目标版本
- 当前所在提交就是要打 tag 的精确提交

## 发布步骤

假设目标版本是 `<version>`。

1. 本地确认发布提交状态：

```bash
cargo check
cargo test
git status --short
```

2. 如果需要，提交并推送发布变更：

```bash
git add Cargo.toml Cargo.lock README.md RELEASING.md RELEASING.zh-CN.md src .github packaging scripts
git commit -m "release: v<version>"
git push origin main
```

3. 给准确的发布提交打 tag 并推送：

```bash
git tag -a v<version> -m "v<version>"
git push origin v<version>
```

4. 等待 GitHub Actions 的 `release` workflow 跑完。

GitHub Release 中应该出现这些资源文件：

- `wetwin-aarch64-apple-darwin.tar.gz`
- `wetwin-x86_64-apple-darwin.tar.gz`
- `wetwin-aarch64.pkg`
- `wetwin-x86_64.pkg`

如果 workflow 没有自动触发，就手动以 `v<version>` 为 tag 触发一次。

5. 重新生成仓库内打包好的 Homebrew formula：

```bash
./scripts/update-homebrew-formula.sh <version>
```

6. 提交当前仓库中的 formula 样板更新：

```bash
git add packaging/homebrew-tap/Formula/wetwin.rb scripts/update-homebrew-formula.sh .github/workflows/release.yml
git commit -m "chore: refresh packaged Homebrew formula"
git push origin main
```

7. 把 formula 复制到 tap 仓库：

```bash
cp packaging/homebrew-tap/Formula/wetwin.rb ../homebrew-tap/Formula/wetwin.rb
```

8. 发布 tap 仓库更新：

```bash
cd ../homebrew-tap
git add Formula/wetwin.rb README.md README.zh-CN.md
git commit -m "Update wetwin formula for v<version>"
git push origin main
```

9. 验证发布后的安装路径：

```bash
brew update
brew upgrade wetwin
wetwin --version
brew info life2you/tap/wetwin
```

## 注意事项

- 不要在 release 资源生成之前更新 tap 公式。
- 现在的 Homebrew 安装走的是预编译二进制，终端用户机器上不需要再安装 Rust。
- `.pkg` 安装包会把 `wetwin` 安装到 `/usr/local/bin/wetwin`，Apple Silicon 和 Intel 都使用同一个安装路径。
