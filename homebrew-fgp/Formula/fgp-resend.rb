# typed: false
# frozen_string_literal: true

class FgpResend < Formula
  desc "FGP Resend daemon - Fast transactional email via Resend API"
  homepage "https://github.com/fast-gateway-protocol/resend"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/resend/releases/download/v#{version}/fgp-resend-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/resend/releases/download/v#{version}/fgp-resend-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/resend/releases/download/v#{version}/fgp-resend-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-resend"
    (var/"fgp/services/resend").mkpath
  end

  def caveats
    <<~EOS
      Resend daemon requires API key setup.

      Set environment variable:
        export RESEND_API_KEY="re_..."

      Quick start:
        fgp start resend                                  # Start daemon
        fgp call resend send --to user@example.com "Hi"   # Send email
        fgp call resend domains                           # List domains

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/resend/
    EOS
  end

  service do
    run [opt_bin/"fgp-resend", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/resend"
    log_path var/"log/fgp-resend.log"
    error_log_path var/"log/fgp-resend.log"
  end

  test do
    assert_match "fgp-resend", shell_output("#{bin}/fgp-resend --version")
  end
end
