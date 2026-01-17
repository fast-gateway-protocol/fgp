# typed: false
# frozen_string_literal: true

class FgpNeon < Formula
  desc "FGP Neon daemon - Fast Neon Postgres operations"
  homepage "https://github.com/fast-gateway-protocol/neon"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/neon/releases/download/v#{version}/fgp-neon-macos-arm64.tar.gz"
      sha256 "56fb985b1cf480123c0563c42d78fed14d836f9108b7e73a6f6c79acb6e75cc0"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/neon/releases/download/v#{version}/fgp-neon-macos-x64.tar.gz"
      sha256 "bf468b5a203dd73d43ae75f6f2a9ae0a38bc6b20c9f00ebac613c50ca1f46429"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/fast-gateway-protocol/neon/releases/download/v#{version}/fgp-neon-linux-arm64.tar.gz"
      sha256 "783fcd35518faf3f1ceeea0f4f389e37721a62c2c9589a708ee7b5dcd18576b0"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/neon/releases/download/v#{version}/fgp-neon-linux-x64.tar.gz"
      sha256 "2b0048bbf9bf14ce8f9a053d20f5ac9dfd5d78a3872986403b6a0806d17de28a"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-neon"
    (var/"fgp/services/neon").mkpath
  end

  def caveats
    <<~EOS
      Neon daemon requires Neon API key.
      Set NEON_API_KEY environment variable.

      Quick start:
        fgp start neon                     # Start daemon
        fgp call neon projects             # List projects
        fgp call neon query "SELECT 1"     # Run SQL

      Documentation: https://fast-gateway-protocol.github.io/fgp/
    EOS
  end

  service do
    run [opt_bin/"fgp-neon", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/neon"
    log_path var/"log/fgp-neon.log"
    error_log_path var/"log/fgp-neon.log"
  end

  test do
    assert_match "fgp-neon", shell_output("#{bin}/fgp-neon --version")
  end
end
