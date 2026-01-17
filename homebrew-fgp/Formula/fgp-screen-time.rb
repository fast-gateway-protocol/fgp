# typed: false
# frozen_string_literal: true

class FgpScreenTime < Formula
  desc "FGP Screen Time daemon - Fast macOS Screen Time stats via knowledgeC.db"
  homepage "https://github.com/fast-gateway-protocol/screen-time"
  license "MIT"
  version "0.1.0"

  depends_on :macos

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/screen-time/releases/download/v#{version}/fgp-screen-time-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/screen-time/releases/download/v#{version}/fgp-screen-time-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-screen-time"
    (var/"fgp/services/screen-time").mkpath
  end

  def caveats
    <<~EOS
      Screen Time daemon requires Full Disk Access for knowledgeC.db.

      Grant access in System Preferences > Privacy & Security > Full Disk Access.

      Quick start:
        fgp start screen-time                # Start daemon
        fgp call screen-time today           # Today's usage
        fgp call screen-time week            # Weekly breakdown

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/screen-time/
    EOS
  end

  service do
    run [opt_bin/"fgp-screen-time", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/screen-time"
    log_path var/"log/fgp-screen-time.log"
    error_log_path var/"log/fgp-screen-time.log"
  end

  test do
    assert_match "fgp-screen-time", shell_output("#{bin}/fgp-screen-time --version")
  end
end
