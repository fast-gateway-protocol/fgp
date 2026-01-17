# typed: false
# frozen_string_literal: true

class FgpFfmpeg < Formula
  desc "FGP FFmpeg daemon - Fast video/audio processing"
  homepage "https://github.com/fast-gateway-protocol/ffmpeg"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/ffmpeg/releases/download/v#{version}/fgp-ffmpeg-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/ffmpeg/releases/download/v#{version}/fgp-ffmpeg-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/ffmpeg/releases/download/v#{version}/fgp-ffmpeg-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"
  depends_on "ffmpeg"

  def install
    bin.install "fgp-ffmpeg"
    (var/"fgp/services/ffmpeg").mkpath
  end

  def caveats
    <<~EOS
      FFmpeg daemon wraps your system ffmpeg for fast video/audio processing.

      Quick start:
        fgp start ffmpeg                           # Start daemon
        fgp call ffmpeg info video.mp4             # Get video info
        fgp call ffmpeg convert video.avi --to mp4 # Convert format
        fgp call ffmpeg trim video.mp4 --start 10 --end 30

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/ffmpeg/
    EOS
  end

  service do
    run [opt_bin/"fgp-ffmpeg", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/ffmpeg"
    log_path var/"log/fgp-ffmpeg.log"
    error_log_path var/"log/fgp-ffmpeg.log"
  end

  test do
    assert_match "fgp-ffmpeg", shell_output("#{bin}/fgp-ffmpeg --version")
  end
end
