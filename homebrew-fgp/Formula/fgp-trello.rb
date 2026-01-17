# typed: false
# frozen_string_literal: true

class FgpTrello < Formula
  desc "FGP Trello daemon - Fast Trello boards, lists, and cards"
  homepage "https://github.com/fast-gateway-protocol/trello"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/trello/releases/download/v#{version}/fgp-trello-macos-arm64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_ARM64"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/trello/releases/download/v#{version}/fgp-trello-macos-x64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_X64"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/fast-gateway-protocol/trello/releases/download/v#{version}/fgp-trello-linux-arm64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/trello/releases/download/v#{version}/fgp-trello-linux-x64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X64"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-trello"
    (var/"fgp/services/trello").mkpath
  end

  def caveats
    <<~EOS
      Trello daemon requires API key and token.
      Set TRELLO_API_KEY and TRELLO_TOKEN environment variables.
      Get credentials at: https://trello.com/power-ups/admin

      Quick start:
        fgp start trello                      # Start daemon
        fgp call trello boards                # List boards
        fgp call trello cards --board "Work"  # List cards

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/trello/
    EOS
  end

  service do
    run [opt_bin/"fgp-trello", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/trello"
    log_path var/"log/fgp-trello.log"
    error_log_path var/"log/fgp-trello.log"
  end

  test do
    assert_match "fgp-trello", shell_output("#{bin}/fgp-trello --version")
  end
end
