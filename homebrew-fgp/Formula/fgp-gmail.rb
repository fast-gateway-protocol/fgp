# typed: false
# frozen_string_literal: true

class FgpGmail < Formula
  desc "FGP Gmail daemon - Fast email operations via Google API"
  homepage "https://github.com/fast-gateway-protocol/gmail"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/gmail/releases/download/v#{version}/fgp-gmail-macos-arm64.tar.gz"
      sha256 "575bf815ca63e3698fd2c509fcae1b3df5d7aa908d7c9655ef6e880decf4cb9d"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/gmail/releases/download/v#{version}/fgp-gmail-macos-x64.tar.gz"
      sha256 "e337c7c982d80220748fab8eeb04665befdf5cc299ce3db2aab490557d302439"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/gmail/releases/download/v#{version}/fgp-gmail-linux-x64.tar.gz"
      sha256 "86051ec734310e067b75d6d7eb910efefbf0867377e20324757cedd98089cc37"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-gmail"
    (var/"fgp/services/gmail").mkpath
  end

  def caveats
    <<~EOS
      Gmail daemon requires Google OAuth setup on first use.

      Quick start:
        fgp start gmail                    # Start daemon
        fgp call gmail inbox               # List inbox
        fgp call gmail search "from:boss"  # Search emails

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/gmail/
    EOS
  end

  service do
    run [opt_bin/"fgp-gmail", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/gmail"
    log_path var/"log/fgp-gmail.log"
    error_log_path var/"log/fgp-gmail.log"
  end

  test do
    assert_match "fgp-gmail", shell_output("#{bin}/fgp-gmail --version")
  end
end
