# typed: false
# frozen_string_literal: true

class FgpYtdlp < Formula
  desc "FGP yt-dlp daemon - Fast video/audio downloading"
  homepage "https://github.com/fast-gateway-protocol/ytdlp"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/ytdlp/releases/download/v#{version}/fgp-ytdlp-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/ytdlp/releases/download/v#{version}/fgp-ytdlp-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/ytdlp/releases/download/v#{version}/fgp-ytdlp-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"
  depends_on "yt-dlp"

  def install
    bin.install "fgp-ytdlp"
    (var/"fgp/services/ytdlp").mkpath
  end

  def caveats
    <<~EOS
      yt-dlp daemon wraps your system yt-dlp for fast video downloading.

      For best quality, also install ffmpeg:
        brew install ffmpeg

      Quick start:
        fgp start ytdlp                          # Start daemon
        fgp call ytdlp download "URL"            # Download video
        fgp call ytdlp audio "URL"               # Extract audio
        fgp call ytdlp info "URL"                # Get metadata

      Supports 1000+ sites including YouTube, Vimeo, Twitter, TikTok, etc.

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/ytdlp/
    EOS
  end

  service do
    run [opt_bin/"fgp-ytdlp", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/ytdlp"
    log_path var/"log/fgp-ytdlp.log"
    error_log_path var/"log/fgp-ytdlp.log"
  end

  test do
    assert_match "fgp-ytdlp", shell_output("#{bin}/fgp-ytdlp --version")
  end
end
