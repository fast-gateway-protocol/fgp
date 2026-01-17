# typed: false
# frozen_string_literal: true

class FgpPostmark < Formula
  desc "FGP Postmark daemon - Fast transactional email with delivery tracking via Postmark API"
  homepage "https://github.com/fast-gateway-protocol/postmark"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/postmark/releases/download/v1.0.0/fgp-postmark-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/postmark/releases/download/v1.0.0/fgp-postmark-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/postmark/releases/download/v1.0.0/fgp-postmark-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-postmark"
    (var/"fgp/services/postmark").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        export POSTMARK_SERVER_TOKEN="your-server-token"
        fgp start postmark
        fgp call postmark.send --to "user@example.com" --subject "Hello" --body "World"
    EOS
  end

  service do
    run [opt_bin/"fgp-postmark", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/postmark"
    log_path var/"log/fgp-postmark.log"
  end

  test do
    assert_match "fgp-postmark", shell_output("#{bin}/fgp-postmark --version")
  end
end
