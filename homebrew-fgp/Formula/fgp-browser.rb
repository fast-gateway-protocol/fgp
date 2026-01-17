# typed: false
# frozen_string_literal: true

class FgpBrowser < Formula
  desc "FGP browser daemon - Chrome automation via DevTools Protocol (292x faster than Playwright MCP)"
  homepage "https://github.com/fast-gateway-protocol/browser"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/browser/releases/download/v#{version}/browser-gateway-macos-arm64.tar.gz"
      sha256 "ef223284d795aead0b9d679a0cc2b9838c3efd05e7c7e55e4fce773e048f67c2"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/browser/releases/download/v#{version}/browser-gateway-macos-x64.tar.gz"
      sha256 "87d24b6e401b62ee5a78d0868458c3b57b4e05bb5570f5442fb4d75ce3553a5f"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/fast-gateway-protocol/browser/releases/download/v#{version}/browser-gateway-linux-arm64.tar.gz"
      sha256 "7a6bc0e4956dda07f008c28a0f6838eb535f0edcf0a00670be11248137f3b7c2"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/browser/releases/download/v#{version}/browser-gateway-linux-x64.tar.gz"
      sha256 "e8930b6ee855e91e83dcb3b76c1c92cecbd9f1665b96a89afa5ff1de0b790b9d"
    end
  end

  depends_on "fgp"

  def install
    bin.install "browser-gateway"

    # Create service directory
    (var/"fgp/services/browser").mkpath
  end

  def post_install
    ohai "FGP Browser daemon installed!"
    ohai "Start with: fgp start browser"
  end

  def caveats
    <<~EOS
      The browser daemon requires Chrome or Chromium to be installed.

      Quick start:
        fgp start browser              # Start daemon (headless)
        fgp start browser --no-headless # Start with visible browser

        fgp call browser open "https://example.com"
        fgp call browser snapshot
        fgp call browser screenshot /tmp/page.png

      Performance: 8ms navigate (vs 2,328ms Playwright MCP) - 292x faster!

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/browser/
    EOS
  end

  service do
    run [opt_bin/"browser-gateway", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/browser"
    log_path var/"log/fgp-browser.log"
    error_log_path var/"log/fgp-browser.log"
  end

  test do
    assert_match "browser-gateway", shell_output("#{bin}/browser-gateway --version")
  end
end
