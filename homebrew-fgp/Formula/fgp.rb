# typed: false
# frozen_string_literal: true

class Fgp < Formula
  desc "Fast Gateway Protocol - Daemon-based architecture for AI agent tools"
  homepage "https://github.com/fast-gateway-protocol/fgp"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/cli/releases/download/v#{version}/fgp-macos-arm64.tar.gz"
      sha256 "9a62bfee4e1f3f5c997365754ac3a0096eb1517564550edf89da41dc19535894"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/cli/releases/download/v#{version}/fgp-macos-x64.tar.gz"
      sha256 "90a6c7f04de187a2cb27189cd3d2a30dde7a9b492435d430878bfe2feea41733"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/fast-gateway-protocol/cli/releases/download/v#{version}/fgp-linux-arm64.tar.gz"
      sha256 "6d16de4dcc56f4ad577d79366e07619d01310863a543efac3aeb9ae382b52e77"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/cli/releases/download/v#{version}/fgp-linux-x64.tar.gz"
      sha256 "9ce31f80214c10756dfc3838f40c56454269e711a13f4b9f5ff68fe54e60f3ef"
    end
  end

  def install
    bin.install "fgp"

    # Create FGP home directory structure
    (var/"fgp/services").mkpath
  end

  def post_install
    ohai "FGP installed successfully!"
    ohai "Quick start:"
    ohai "  fgp start browser    # Start browser daemon"
    ohai "  fgp status           # Check daemon status"
    ohai "  fgp --help           # See all commands"
  end

  def caveats
    <<~EOS
      FGP daemons store their sockets in ~/.fgp/services/

      To install the browser daemon:
        brew install fgp-browser

      Or install all daemons at once:
        brew install fgp-browser fgp-gmail fgp-calendar fgp-github

      Documentation: https://fast-gateway-protocol.github.io/fgp/
    EOS
  end

  test do
    assert_match "fgp", shell_output("#{bin}/fgp --version")
  end
end
