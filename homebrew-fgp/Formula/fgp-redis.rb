# typed: false
# frozen_string_literal: true

class FgpRedis < Formula
  desc "FGP Redis daemon - Fast caching and data structure operations"
  homepage "https://github.com/fast-gateway-protocol/redis"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/redis/releases/download/v#{version}/fgp-redis-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/redis/releases/download/v#{version}/fgp-redis-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/redis/releases/download/v#{version}/fgp-redis-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-redis"
    (var/"fgp/services/redis").mkpath
  end

  def caveats
    <<~EOS
      Redis daemon provides ultra-fast cache and data structure operations.

      Set connection URL:
        export REDIS_URL="redis://localhost:6379"

      Quick start:
        fgp start redis                        # Start daemon
        fgp call redis set mykey "value"       # Set key
        fgp call redis get mykey               # Get key
        fgp call redis hset user:1 name "John" # Hash operations

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/redis/
    EOS
  end

  service do
    run [opt_bin/"fgp-redis", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/redis"
    log_path var/"log/fgp-redis.log"
    error_log_path var/"log/fgp-redis.log"
  end

  test do
    assert_match "fgp-redis", shell_output("#{bin}/fgp-redis --version")
  end
end
