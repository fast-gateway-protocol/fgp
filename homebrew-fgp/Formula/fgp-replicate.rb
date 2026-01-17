# typed: false
# frozen_string_literal: true

class FgpReplicate < Formula
  desc "FGP Replicate daemon - Fast access to ML models on Replicate"
  homepage "https://github.com/fast-gateway-protocol/replicate"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/replicate/releases/download/v1.0.0/fgp-replicate-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/replicate/releases/download/v1.0.0/fgp-replicate-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/replicate/releases/download/v1.0.0/fgp-replicate-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-replicate"
    (var/"fgp/services/replicate").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        export REPLICATE_API_TOKEN="your-api-token"
        fgp start replicate
        fgp call replicate.run --model "stability-ai/sdxl" --input '{"prompt": "a photo of an astronaut"}'
    EOS
  end

  service do
    run [opt_bin/"fgp-replicate", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/replicate"
    log_path var/"log/fgp-replicate.log"
  end

  test do
    assert_match "fgp-replicate", shell_output("#{bin}/fgp-replicate --version")
  end
end
