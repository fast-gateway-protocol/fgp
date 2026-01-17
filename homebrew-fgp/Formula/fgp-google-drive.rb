# typed: false
# frozen_string_literal: true

class FgpGoogleDrive < Formula
  desc "FGP Google Drive daemon - Fast file operations via Google API"
  homepage "https://github.com/fast-gateway-protocol/google-drive"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/google-drive/releases/download/v#{version}/fgp-google-drive-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/google-drive/releases/download/v#{version}/fgp-google-drive-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/google-drive/releases/download/v#{version}/fgp-google-drive-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-google-drive"
    (var/"fgp/services/google-drive").mkpath
  end

  def caveats
    <<~EOS
      Google Drive daemon requires OAuth setup on first use.

      Quick start:
        fgp start google-drive                # Start daemon
        fgp call google-drive auth            # Authenticate
        fgp call google-drive list            # List files
        fgp call google-drive upload file.pdf # Upload file

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/google-drive/
    EOS
  end

  service do
    run [opt_bin/"fgp-google-drive", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/google-drive"
    log_path var/"log/fgp-google-drive.log"
    error_log_path var/"log/fgp-google-drive.log"
  end

  test do
    assert_match "fgp-google-drive", shell_output("#{bin}/fgp-google-drive --version")
  end
end
