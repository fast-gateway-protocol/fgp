# typed: false
# frozen_string_literal: true

class FgpGoogleSheets < Formula
  desc "FGP Google Sheets daemon - Fast spreadsheet operations via Google API"
  homepage "https://github.com/fast-gateway-protocol/google-sheets"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/google-sheets/releases/download/v#{version}/fgp-google-sheets-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/google-sheets/releases/download/v#{version}/fgp-google-sheets-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/google-sheets/releases/download/v#{version}/fgp-google-sheets-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-google-sheets"
    (var/"fgp/services/google-sheets").mkpath
  end

  def caveats
    <<~EOS
      Google Sheets daemon requires OAuth setup on first use.

      Quick start:
        fgp start google-sheets                      # Start daemon
        fgp call google-sheets auth                  # Authenticate
        fgp call google-sheets read <sheet-id>       # Read spreadsheet
        fgp call google-sheets append <id> '[[...]]' # Append rows

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/google-sheets/
    EOS
  end

  service do
    run [opt_bin/"fgp-google-sheets", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/google-sheets"
    log_path var/"log/fgp-google-sheets.log"
    error_log_path var/"log/fgp-google-sheets.log"
  end

  test do
    assert_match "fgp-google-sheets", shell_output("#{bin}/fgp-google-sheets --version")
  end
end
