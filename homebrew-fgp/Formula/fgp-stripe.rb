# typed: false
# frozen_string_literal: true

class FgpStripe < Formula
  desc "FGP Stripe daemon - Fast payments and subscriptions via Stripe API"
  homepage "https://github.com/fast-gateway-protocol/stripe"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/stripe/releases/download/v#{version}/fgp-stripe-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/stripe/releases/download/v#{version}/fgp-stripe-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/stripe/releases/download/v#{version}/fgp-stripe-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-stripe"
    (var/"fgp/services/stripe").mkpath
  end

  def caveats
    <<~EOS
      Stripe daemon requires API key setup.

      Set environment variable:
        export STRIPE_SECRET_KEY="sk_..."

      Quick start:
        fgp start stripe                         # Start daemon
        fgp call stripe customers                # List customers
        fgp call stripe subscriptions            # List subscriptions

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/stripe/
    EOS
  end

  service do
    run [opt_bin/"fgp-stripe", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/stripe"
    log_path var/"log/fgp-stripe.log"
    error_log_path var/"log/fgp-stripe.log"
  end

  test do
    assert_match "fgp-stripe", shell_output("#{bin}/fgp-stripe --version")
  end
end
