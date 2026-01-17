# typed: false
# frozen_string_literal: true

class FgpPusher < Formula
  desc "FGP Pusher daemon - Fast real-time messaging and WebSocket channels via Pusher API"
  homepage "https://github.com/fast-gateway-protocol/pusher"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/pusher/releases/download/v1.0.0/fgp-pusher-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/pusher/releases/download/v1.0.0/fgp-pusher-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/pusher/releases/download/v1.0.0/fgp-pusher-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-pusher"
    (var/"fgp/services/pusher").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        export PUSHER_APP_ID="your-app-id"
        export PUSHER_KEY="your-key"
        export PUSHER_SECRET="your-secret"
        export PUSHER_CLUSTER="your-cluster"
        fgp start pusher
        fgp call pusher.trigger --channel "my-channel" --event "my-event" --data '{"message":"hello"}'
    EOS
  end

  service do
    run [opt_bin/"fgp-pusher", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/pusher"
    log_path var/"log/fgp-pusher.log"
  end

  test do
    assert_match "fgp-pusher", shell_output("#{bin}/fgp-pusher --version")
  end
end
