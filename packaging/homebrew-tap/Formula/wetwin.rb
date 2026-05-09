class Wetwin < Formula
  desc "Lightweight macOS WeChat multi-instance manager with a terminal UI"
  homepage "https://github.com/life2you/wetwin"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/life2you/wetwin/releases/download/v0.1.0/wetwin-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ARM64_SHA256"
    end

    on_intel do
      url "https://github.com/life2you/wetwin/releases/download/v0.1.0/wetwin-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_X64_SHA256"
    end
  end

  def install
    bin.install "wetwin"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/wetwin --version")
  end
end
