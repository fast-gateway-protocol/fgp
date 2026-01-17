# typed: false
# frozen_string_literal: true

class FgpAnthropic < Formula
  desc "FGP Anthropic daemon - Fast Claude API operations"
  homepage "https://github.com/fast-gateway-protocol/anthropic"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/anthropic/releases/download/v1.0.0/fgp-anthropic-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/anthropic/releases/download/v1.0.0/fgp-anthropic-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/anthropic/releases/download/v1.0.0/fgp-anthropic-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-anthropic"
    (var/"fgp/services/anthropic").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        export ANTHROPIC_API_KEY="sk-ant-..."
        fgp start anthropic
        fgp call anthropic.chat --model claude-sonnet-4-20250514 --message "Hello!"
    EOS
  end

  service do
    run [opt_bin/"fgp-anthropic", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/anthropic"
    log_path var/"log/fgp-anthropic.log"
  end

  test do
    assert_match "fgp-anthropic", shell_output("#{bin}/fgp-anthropic --version")
  end
end
