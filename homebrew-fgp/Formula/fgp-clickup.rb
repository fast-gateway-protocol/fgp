# typed: false
# frozen_string_literal: true

class FgpClickup < Formula
  desc "FGP ClickUp daemon - Fast ClickUp tasks, spaces, and folders"
  homepage "https://github.com/fast-gateway-protocol/clickup"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/clickup/releases/download/v#{version}/fgp-clickup-macos-arm64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_ARM64"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/clickup/releases/download/v#{version}/fgp-clickup-macos-x64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_X64"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/fast-gateway-protocol/clickup/releases/download/v#{version}/fgp-clickup-linux-arm64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/clickup/releases/download/v#{version}/fgp-clickup-linux-x64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X64"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-clickup"
    (var/"fgp/services/clickup").mkpath
  end

  def caveats
    <<~EOS
      ClickUp daemon requires a ClickUp API token.
      Set CLICKUP_API_TOKEN environment variable.
      Get your token at: https://app.clickup.com/settings/apps

      Quick start:
        fgp start clickup                       # Start daemon
        fgp call clickup tasks --list "Sprint"  # List tasks
        fgp call clickup create_task "New task" # Create task

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/clickup/
    EOS
  end

  service do
    run [opt_bin/"fgp-clickup", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/clickup"
    log_path var/"log/fgp-clickup.log"
    error_log_path var/"log/fgp-clickup.log"
  end

  test do
    assert_match "fgp-clickup", shell_output("#{bin}/fgp-clickup --version")
  end
end
