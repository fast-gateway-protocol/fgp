# typed: false
# frozen_string_literal: true

class FgpMonday < Formula
  desc "FGP Monday daemon - Fast Monday.com boards, items, and columns"
  homepage "https://github.com/fast-gateway-protocol/monday"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/monday/releases/download/v#{version}/fgp-monday-macos-arm64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_ARM64"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/monday/releases/download/v#{version}/fgp-monday-macos-x64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_X64"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/fast-gateway-protocol/monday/releases/download/v#{version}/fgp-monday-linux-arm64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/monday/releases/download/v#{version}/fgp-monday-linux-x64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X64"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-monday"
    (var/"fgp/services/monday").mkpath
  end

  def caveats
    <<~EOS
      Monday daemon requires a Monday.com API token.
      Set MONDAY_API_TOKEN environment variable.
      Get your token at: https://monday.com/developers/v2

      Quick start:
        fgp start monday                       # Start daemon
        fgp call monday boards                 # List boards
        fgp call monday items --board "Tasks"  # List items

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/monday/
    EOS
  end

  service do
    run [opt_bin/"fgp-monday", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/monday"
    log_path var/"log/fgp-monday.log"
    error_log_path var/"log/fgp-monday.log"
  end

  test do
    assert_match "fgp-monday", shell_output("#{bin}/fgp-monday --version")
  end
end
