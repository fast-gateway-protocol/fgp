# typed: false
# frozen_string_literal: true

class FgpMailgun < Formula
  desc "FGP Mailgun daemon - Fast email delivery and tracking via Mailgun API"
  homepage "https://github.com/fast-gateway-protocol/mailgun"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/mailgun/releases/download/v1.0.0/fgp-mailgun-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/mailgun/releases/download/v1.0.0/fgp-mailgun-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/mailgun/releases/download/v1.0.0/fgp-mailgun-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-mailgun"
    (var/"fgp/services/mailgun").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        export MAILGUN_API_KEY="your-api-key"
        export MAILGUN_DOMAIN="your-domain.mailgun.org"
        fgp start mailgun
        fgp call mailgun.send --to "user@example.com" --subject "Hello" --body "World"
    EOS
  end

  service do
    run [opt_bin/"fgp-mailgun", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/mailgun"
    log_path var/"log/fgp-mailgun.log"
  end

  test do
    assert_match "fgp-mailgun", shell_output("#{bin}/fgp-mailgun --version")
  end
end
