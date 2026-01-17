# typed: false
# frozen_string_literal: true

class FgpOpenai < Formula
  desc "FGP OpenAI daemon - Fast GPT, DALL-E, and Whisper API access"
  homepage "https://github.com/fast-gateway-protocol/openai"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/openai/releases/download/v1.0.0/fgp-openai-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/openai/releases/download/v1.0.0/fgp-openai-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/openai/releases/download/v1.0.0/fgp-openai-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-openai"
    (var/"fgp/services/openai").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        export OPENAI_API_KEY="your-api-key"
        fgp start openai
        fgp call openai.chat --model "gpt-4" --messages '[{"role": "user", "content": "Hello"}]'
    EOS
  end

  service do
    run [opt_bin/"fgp-openai", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/openai"
    log_path var/"log/fgp-openai.log"
  end

  test do
    assert_match "fgp-openai", shell_output("#{bin}/fgp-openai --version")
  end
end
