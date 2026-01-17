# typed: false
# frozen_string_literal: true

class FgpTwilio < Formula
  desc "FGP Twilio daemon - Fast SMS, calls, and WhatsApp via Twilio API"
  homepage "https://github.com/fast-gateway-protocol/twilio"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/twilio/releases/download/v#{version}/fgp-twilio-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/twilio/releases/download/v#{version}/fgp-twilio-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/twilio/releases/download/v#{version}/fgp-twilio-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-twilio"
    (var/"fgp/services/twilio").mkpath
  end

  def caveats
    <<~EOS
      Twilio daemon requires account credentials.

      Set environment variables:
        export TWILIO_ACCOUNT_SID="AC..."
        export TWILIO_AUTH_TOKEN="..."
        export TWILIO_PHONE_NUMBER="+1..."

      Quick start:
        fgp start twilio                              # Start daemon
        fgp call twilio sms "+1234567890" "Hello"     # Send SMS
        fgp call twilio call "+1234567890"            # Make call

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/twilio/
    EOS
  end

  service do
    run [opt_bin/"fgp-twilio", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/twilio"
    log_path var/"log/fgp-twilio.log"
    error_log_path var/"log/fgp-twilio.log"
  end

  test do
    assert_match "fgp-twilio", shell_output("#{bin}/fgp-twilio --version")
  end
end
