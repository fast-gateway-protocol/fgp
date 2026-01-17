# typed: false
# frozen_string_literal: true

class FgpCalendar < Formula
  desc "FGP Calendar daemon - Fast Google Calendar operations"
  homepage "https://github.com/fast-gateway-protocol/calendar"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/calendar/releases/download/v#{version}/fgp-calendar-macos-arm64.tar.gz"
      sha256 "f89df7c2b56ed96d38f3a86b39a84fd11ba020fd4b70e73d4b4fc26371775382"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/calendar/releases/download/v#{version}/fgp-calendar-macos-x64.tar.gz"
      sha256 "70d922e2d02342fcd357c8d132e75dd561cdeb216265504fa39fdc848551d340"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/calendar/releases/download/v#{version}/fgp-calendar-linux-x64.tar.gz"
      sha256 "29fd5bf9073563f7b2e8a2e294ac7b58ec652b8909ccd14eea4cc174414e82ed"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-calendar"
    (var/"fgp/services/calendar").mkpath
  end

  def caveats
    <<~EOS
      Calendar daemon requires Google OAuth setup on first use.

      Quick start:
        fgp start calendar                 # Start daemon
        fgp call calendar today            # Today's events
        fgp call calendar upcoming         # Next 7 days
        fgp call calendar free_slots       # Find available times

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/calendar/
    EOS
  end

  service do
    run [opt_bin/"fgp-calendar", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/calendar"
    log_path var/"log/fgp-calendar.log"
    error_log_path var/"log/fgp-calendar.log"
  end

  test do
    assert_match "fgp-calendar", shell_output("#{bin}/fgp-calendar --version")
  end
end
