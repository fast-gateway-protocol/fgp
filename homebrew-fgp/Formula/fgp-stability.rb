# typed: false
# frozen_string_literal: true

class FgpStability < Formula
  desc "FGP Stability AI daemon - Fast Stable Diffusion and image generation"
  homepage "https://github.com/fast-gateway-protocol/stability"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/stability/releases/download/v1.0.0/fgp-stability-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/stability/releases/download/v1.0.0/fgp-stability-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/stability/releases/download/v1.0.0/fgp-stability-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-stability"
    (var/"fgp/services/stability").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        export STABILITY_API_KEY="your-api-key"
        fgp start stability
        fgp call stability.generate --prompt "a beautiful sunset over mountains" --output "/tmp/image.png"
    EOS
  end

  service do
    run [opt_bin/"fgp-stability", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/stability"
    log_path var/"log/fgp-stability.log"
  end

  test do
    assert_match "fgp-stability", shell_output("#{bin}/fgp-stability --version")
  end
end
