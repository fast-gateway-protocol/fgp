# typed: false
# frozen_string_literal: true

class FgpNotion < Formula
  desc "FGP Notion daemon - Fast Notion pages, databases, and blocks"
  homepage "https://github.com/fast-gateway-protocol/notion"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/notion/releases/download/v#{version}/fgp-notion-macos-arm64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_ARM64"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/notion/releases/download/v#{version}/fgp-notion-macos-x64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_X64"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/fast-gateway-protocol/notion/releases/download/v#{version}/fgp-notion-linux-arm64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/notion/releases/download/v#{version}/fgp-notion-linux-x64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X64"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-notion"
    (var/"fgp/services/notion").mkpath
  end

  def caveats
    <<~EOS
      Notion daemon requires a Notion integration token.
      Set NOTION_API_KEY environment variable.
      Create an integration at: https://www.notion.so/my-integrations

      Important: Share pages/databases with your integration for access.

      Quick start:
        fgp start notion                        # Start daemon
        fgp call notion search "meeting notes"  # Search pages
        fgp call notion page <page_id>          # Get page

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/notion/
    EOS
  end

  service do
    run [opt_bin/"fgp-notion", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/notion"
    log_path var/"log/fgp-notion.log"
    error_log_path var/"log/fgp-notion.log"
  end

  test do
    assert_match "fgp-notion", shell_output("#{bin}/fgp-notion --version")
  end
end
