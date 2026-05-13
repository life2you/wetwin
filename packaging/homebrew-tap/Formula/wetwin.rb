class Wetwin < Formula
  desc "Lightweight macOS WeChat multi-instance manager with a terminal UI"
  homepage "https://github.com/life2you/wetwin"
  version "0.1.1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/life2you/wetwin/releases/download/v0.1.1/wetwin-aarch64-apple-darwin.tar.gz"
      sha256 "35a4eb9239a1dfad941c8e4ccf344e406870273a52be30427430846ee91245d7"
    end

    on_intel do
      url "https://github.com/life2you/wetwin/releases/download/v0.1.1/wetwin-x86_64-apple-darwin.tar.gz"
      sha256 "02a31157017a08aa7a4dd305e8395cf7a301074ae711dfbd6897da50125c6af8"
    end
  end

  def install
    bin.install "wetwin"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/wetwin --version")
  end
end
