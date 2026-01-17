# typed: false
# frozen_string_literal: true

class FgpSendgrid < Formula
  desc "FGP SendGrid daemon - Fast transactional email delivery via SendGrid API"
  homepage "https://github.com/fast-gateway-protocol/sendgrid"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/sendgrid/releases/download/v1.0.0/fgp-sendgrid-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/sendgrid/releases/download/v1.0.0/fgp-sendgrid-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/sendgrid/releases/download/v1.0.0/fgp-sendgrid-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-sendgrid"
    (var/"fgp/services/sendgrid").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        export SENDGRID_API_KEY="your-api-key"
        fgp start sendgrid
        fgp call sendgrid.send --to "user@example.com" --subject "Hello" --body "World"
    EOS
  end

  service do
    run [opt_bin/"fgp-sendgrid", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/sendgrid"
    log_path var/"log/fgp-sendgrid.log"
  end

  test do
    assert_match "fgp-sendgrid", shell_output("#{bin}/fgp-sendgrid --version")
  end
end
