# typed: false
# frozen_string_literal: true

class FgpCloudflare < Formula
  desc "FGP Cloudflare daemon - Fast DNS, Workers, KV operations"
  homepage "https://github.com/fast-gateway-protocol/cloudflare"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/cloudflare/releases/download/v#{version}/fgp-cloudflare-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/cloudflare/releases/download/v#{version}/fgp-cloudflare-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/cloudflare/releases/download/v#{version}/fgp-cloudflare-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-cloudflare"
    (var/"fgp/services/cloudflare").mkpath
  end

  def caveats
    <<~EOS
      Cloudflare daemon requires API token setup.

      Set environment variable:
        export CLOUDFLARE_API_TOKEN="your-token"

      Quick start:
        fgp start cloudflare              # Start daemon
        fgp call cloudflare dns list      # List DNS records
        fgp call cloudflare kv get mykey  # Get KV value

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/cloudflare/
    EOS
  end

  service do
    run [opt_bin/"fgp-cloudflare", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/cloudflare"
    log_path var/"log/fgp-cloudflare.log"
    error_log_path var/"log/fgp-cloudflare.log"
  end

  test do
    assert_match "fgp-cloudflare", shell_output("#{bin}/fgp-cloudflare --version")
  end
end
