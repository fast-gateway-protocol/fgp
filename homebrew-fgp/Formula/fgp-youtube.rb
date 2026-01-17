# typed: false
# frozen_string_literal: true

class FgpYoutube < Formula
  desc "FGP YouTube daemon - Fast YouTube Data API operations"
  homepage "https://github.com/fast-gateway-protocol/youtube"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/youtube/releases/download/v#{version}/fgp-youtube-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/youtube/releases/download/v#{version}/fgp-youtube-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/youtube/releases/download/v#{version}/fgp-youtube-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-youtube"
    (var/"fgp/services/youtube").mkpath
  end

  def caveats
    <<~EOS
      YouTube daemon requires API key setup.

      Set environment variable:
        export YOUTUBE_API_KEY="..."

      Quick start:
        fgp start youtube                       # Start daemon
        fgp call youtube search "rust tutorial" # Search videos
        fgp call youtube video <video-id>       # Get video info

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/youtube/
    EOS
  end

  service do
    run [opt_bin/"fgp-youtube", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/youtube"
    log_path var/"log/fgp-youtube.log"
    error_log_path var/"log/fgp-youtube.log"
  end

  test do
    assert_match "fgp-youtube", shell_output("#{bin}/fgp-youtube --version")
  end
end
