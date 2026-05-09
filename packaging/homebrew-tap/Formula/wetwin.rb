class Wetwin < Formula
  desc "Lightweight macOS WeChat multi-instance manager with a terminal UI"
  homepage "https://github.com/life2you/wetwin"
  url "https://github.com/life2you/wetwin/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "9f30d8e641450ab0e08213ae75bf27530cad5e152d648c9822a43ccec533f21b"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/wetwin --version")
  end
end
