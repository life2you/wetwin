class Wetwin < Formula
  desc "Lightweight macOS WeChat multi-instance manager with a terminal UI"
  homepage "https://github.com/life2you/wetwin"
  version "0.1.3"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/life2you/wetwin/releases/download/v0.1.3/wetwin-aarch64-apple-darwin.tar.gz"
      sha256 "d1b0616310c10a883c2482df6022ec27787dbd3d0a39494e471852ec35c15e0d"
    end

    on_intel do
      url "https://github.com/life2you/wetwin/releases/download/v0.1.3/wetwin-x86_64-apple-darwin.tar.gz"
      sha256 "2999e4cce979458b355e6c72bfd030bab19b6fb17e41b05dc4e9c713e538d70b"
    end
  end

  def install
    bin.install "wetwin"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/wetwin --version")
  end
end
