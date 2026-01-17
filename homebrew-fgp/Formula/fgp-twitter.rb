# typed: false
# frozen_string_literal: true

class FgpTwitter < Formula
  desc "FGP Twitter daemon - Fast Twitter/X API operations"
  homepage "https://github.com/fast-gateway-protocol/twitter"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/twitter/releases/download/v#{version}/fgp-twitter-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/twitter/releases/download/v#{version}/fgp-twitter-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/twitter/releases/download/v#{version}/fgp-twitter-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-twitter"
    (var/"fgp/services/twitter").mkpath
  end

  def caveats
    <<~EOS
      Twitter daemon requires API credentials.

      Set environment variables:
        export TWITTER_API_KEY="..."
        export TWITTER_API_SECRET="..."
        export TWITTER_ACCESS_TOKEN="..."
        export TWITTER_ACCESS_SECRET="..."

      Quick start:
        fgp start twitter                    # Start daemon
        fgp call twitter timeline            # Get timeline
        fgp call twitter post "Hello world"  # Post tweet

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/twitter/
    EOS
  end

  service do
    run [opt_bin/"fgp-twitter", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/twitter"
    log_path var/"log/fgp-twitter.log"
    error_log_path var/"log/fgp-twitter.log"
  end

  test do
    assert_match "fgp-twitter", shell_output("#{bin}/fgp-twitter --version")
  end
end
