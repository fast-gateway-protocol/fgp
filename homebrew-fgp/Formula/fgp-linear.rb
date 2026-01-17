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
      sha256 "81198496c66b9099b48d3a5fb1efcf12b9bd911c2505532495945514ff97007f"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/linear/releases/download/v#{version}/fgp-linear-macos-x64.tar.gz"
      sha256 "5ca8fbc45dec426bfddc14a8e969052061a1cb2a844facc7083cb54429224bec"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/fast-gateway-protocol/linear/releases/download/v#{version}/fgp-linear-linux-arm64.tar.gz"
      sha256 "743c4eb65123cf55e1f46e7a2d3472e423f8544147531bb5e951551b1d4e7dc9"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/linear/releases/download/v#{version}/fgp-linear-linux-x64.tar.gz"
      sha256 "159577ad773eaeab7a65d9f18e99b4f704e2ce879ba1e35fede21fb6e6806549"
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
