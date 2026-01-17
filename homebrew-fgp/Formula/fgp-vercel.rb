# typed: false
# frozen_string_literal: true

class FgpVercel < Formula
  desc "FGP Vercel daemon - Fast Vercel deployment operations"
  homepage "https://github.com/fast-gateway-protocol/vercel"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/vercel/releases/download/v#{version}/fgp-vercel-macos-arm64.tar.gz"
      sha256 "d0e62cde6f609d47eab25c3b200a14a6d06ff9682c4b6613ef8a1c5ab731b52d"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/vercel/releases/download/v#{version}/fgp-vercel-macos-x64.tar.gz"
      sha256 "e305c7ef0074eb64237556d12e475933cc7fbd23c25aa45a3111a21b80d3132a"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/fast-gateway-protocol/vercel/releases/download/v#{version}/fgp-vercel-linux-arm64.tar.gz"
      sha256 "45a3af6021507e748b08b62d3e60fe4d016378c67fa9e8e9643faaa97ed7b75a"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/vercel/releases/download/v#{version}/fgp-vercel-linux-x64.tar.gz"
      sha256 "319d6d10d409a22d455fc927b6dfeb9af4505969929b91f382441d7da8217f49"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-vercel"
    (var/"fgp/services/vercel").mkpath
  end

  def caveats
    <<~EOS
      Vercel daemon uses `vercel` CLI authentication.
      Run `vercel login` if not already authenticated.

      Quick start:
        fgp start vercel                   # Start daemon
        fgp call vercel projects           # List projects
        fgp call vercel deployments        # List deployments

      Documentation: https://fast-gateway-protocol.github.io/fgp/
    EOS
  end

  service do
    run [opt_bin/"fgp-vercel", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/vercel"
    log_path var/"log/fgp-vercel.log"
    error_log_path var/"log/fgp-vercel.log"
  end

  test do
    assert_match "fgp-vercel", shell_output("#{bin}/fgp-vercel --version")
  end
end
