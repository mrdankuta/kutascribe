class Kutascribe < Formula
  desc "Local speech-to-text (audio+video to .txt), single binary"
  homepage "https://github.com/mrdankuta/kutascribe"
  url "https://github.com/mrdankuta/kutascribe/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_RELEASE_TARBALL_SHA256"
  license "MIT"

  depends_on "rust" => :build
  depends_on "ffmpeg"
  depends_on "pkg-config" => :build

  def install
    system "cargo", "install", "--locked", "--root", prefix, "--path", "rust", "--bin", "kutascribe", "--no-default-features"
    # GPU on macOS via Metal needs --features metal (when added); Linux brew gets CPU portable by default
  end

  test do
    system "#{bin}/kutascribe", "--help"
  end
end
