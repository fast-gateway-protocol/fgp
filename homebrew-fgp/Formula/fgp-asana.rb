# typed: false
# frozen_string_literal: true

class FgpAsana < Formula
  desc "FGP Asana daemon - Fast Asana tasks, projects, and workspaces"
  homepage "https://github.com/fast-gateway-protocol/asana"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/asana/releases/download/v#{version}/fgp-asana-macos-arm64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_ARM64"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/asana/releases/download/v#{version}/fgp-asana-macos-x64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_X64"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/fast-gateway-protocol/asana/releases/download/v#{version}/fgp-asana-linux-arm64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/asana/releases/download/v#{version}/fgp-asana-linux-x64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X64"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-asana"
    (var/"fgp/services/asana").mkpath
  end

  def caveats
    <<~EOS
      Asana daemon requires an Asana personal access token.
      Set ASANA_ACCESS_TOKEN environment variable.
      Create a token at: https://app.asana.com/0/developer-console

      Quick start:
        fgp start asana                          # Start daemon
        fgp call asana tasks --project "Sprint"  # List tasks
        fgp call asana create_task "New task"    # Create task

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/asana/
    EOS
  end

  service do
    run [opt_bin/"fgp-asana", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/asana"
    log_path var/"log/fgp-asana.log"
    error_log_path var/"log/fgp-asana.log"
  end

  test do
    assert_match "fgp-asana", shell_output("#{bin}/fgp-asana --version")
  end
end
