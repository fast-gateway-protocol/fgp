# typed: false
# frozen_string_literal: true

class FgpSpotify < Formula
  desc "FGP Spotify daemon - Fast playback control and playlist management"
  homepage "https://github.com/fast-gateway-protocol/spotify"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/spotify/releases/download/v1.0.0/fgp-spotify-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/spotify/releases/download/v1.0.0/fgp-spotify-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/spotify/releases/download/v1.0.0/fgp-spotify-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-spotify"
    (var/"fgp/services/spotify").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        fgp-spotify auth  # OAuth setup
        fgp start spotify
        fgp call spotify.now_playing
    EOS
  end

  service do
    run [opt_bin/"fgp-spotify", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/spotify"
    log_path var/"log/fgp-spotify.log"
  end

  test do
    assert_match "fgp-spotify", shell_output("#{bin}/fgp-spotify --version")
  end
end
