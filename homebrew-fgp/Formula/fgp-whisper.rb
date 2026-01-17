# typed: false
# frozen_string_literal: true

class FgpWhisper < Formula
  desc "FGP Whisper daemon - Fast audio transcription"
  homepage "https://github.com/fast-gateway-protocol/whisper"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/whisper/releases/download/v#{version}/fgp-whisper-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/whisper/releases/download/v#{version}/fgp-whisper-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/whisper/releases/download/v#{version}/fgp-whisper-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"
  depends_on "ffmpeg"

  def install
    bin.install "fgp-whisper"
    (var/"fgp/services/whisper").mkpath
    (var/"fgp/models/whisper").mkpath
  end

  def caveats
    <<~EOS
      Whisper daemon provides fast audio transcription.

      For local inference, install a whisper backend:
        brew install whisper-cpp   (recommended, uses Metal on Apple Silicon)
        pip install openai-whisper (alternative)

      Or use OpenAI API:
        export OPENAI_API_KEY="sk-..."

      Quick start:
        fgp start whisper                         # Start daemon
        fgp call whisper transcribe audio.mp3     # Transcribe audio
        fgp call whisper translate video.mp4      # Translate to English
        fgp call whisper transcribe audio.mp3 --format srt  # Generate subtitles

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/whisper/
    EOS
  end

  service do
    run [opt_bin/"fgp-whisper", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/whisper"
    log_path var/"log/fgp-whisper.log"
    error_log_path var/"log/fgp-whisper.log"
  end

  test do
    assert_match "fgp-whisper", shell_output("#{bin}/fgp-whisper --version")
  end
end
