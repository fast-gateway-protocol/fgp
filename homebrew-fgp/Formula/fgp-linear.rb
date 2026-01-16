# typed: false
# frozen_string_literal: true

class FgpLinear < Formula
  desc "FGP Linear daemon - Fast Linear issue tracking via GraphQL"
  homepage "https://github.com/fast-gateway-protocol/linear"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/linear/releases/download/v#{version}/fgp-linear-macos-arm64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_ARM64"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/linear/releases/download/v#{version}/fgp-linear-macos-x64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_X64"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/fast-gateway-protocol/linear/releases/download/v#{version}/fgp-linear-linux-arm64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/linear/releases/download/v#{version}/fgp-linear-linux-x64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X64"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-linear"
    (var/"fgp/services/linear").mkpath
  end

  def caveats
    <<~EOS
      Linear daemon requires a Linear API key.
      Set LINEAR_API_KEY environment variable.
      Create a key at: https://linear.app/settings/api

      Quick start:
        fgp start linear                      # Start daemon
        fgp call linear me                    # Get current user
        fgp call linear issues --team "Eng"   # List issues

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/linear/
    EOS
  end

  service do
    run [opt_bin/"fgp-linear", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/linear"
    log_path var/"log/fgp-linear.log"
    error_log_path var/"log/fgp-linear.log"
  end

  test do
    assert_match "fgp-linear", shell_output("#{bin}/fgp-linear --version")
  end
end
