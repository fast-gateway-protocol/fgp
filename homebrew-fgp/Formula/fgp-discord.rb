# typed: false
# frozen_string_literal: true

class FgpDiscord < Formula
  desc "FGP Discord daemon - Fast Discord bot operations"
  homepage "https://github.com/fast-gateway-protocol/discord"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/discord/releases/download/v#{version}/fgp-discord-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/discord/releases/download/v#{version}/fgp-discord-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/discord/releases/download/v#{version}/fgp-discord-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-discord"
    (var/"fgp/services/discord").mkpath
  end

  def caveats
    <<~EOS
      Discord daemon requires bot token setup.

      Set environment variable:
        export DISCORD_BOT_TOKEN="your-token"

      Quick start:
        fgp start discord                    # Start daemon
        fgp call discord send #general "Hi"  # Send message
        fgp call discord guilds              # List servers

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/discord/
    EOS
  end

  service do
    run [opt_bin/"fgp-discord", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/discord"
    log_path var/"log/fgp-discord.log"
    error_log_path var/"log/fgp-discord.log"
  end

  test do
    assert_match "fgp-discord", shell_output("#{bin}/fgp-discord --version")
  end
end
