# typed: false
# frozen_string_literal: true

class FgpIntercom < Formula
  desc "FGP Intercom daemon - Fast customer messaging and support via Intercom API"
  homepage "https://github.com/fast-gateway-protocol/intercom"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/intercom/releases/download/v1.0.0/fgp-intercom-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/intercom/releases/download/v1.0.0/fgp-intercom-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/intercom/releases/download/v1.0.0/fgp-intercom-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-intercom"
    (var/"fgp/services/intercom").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        export INTERCOM_ACCESS_TOKEN="your-access-token"
        fgp start intercom
        fgp call intercom.send_message --user_id "12345" --body "Hello, how can I help?"
    EOS
  end

  service do
    run [opt_bin/"fgp-intercom", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/intercom"
    log_path var/"log/fgp-intercom.log"
  end

  test do
    assert_match "fgp-intercom", shell_output("#{bin}/fgp-intercom --version")
  end
end
