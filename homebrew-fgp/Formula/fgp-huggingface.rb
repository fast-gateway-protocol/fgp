# typed: false
# frozen_string_literal: true

class FgpHuggingface < Formula
  desc "FGP Hugging Face daemon - Fast inference API and model hub access"
  homepage "https://github.com/fast-gateway-protocol/huggingface"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/huggingface/releases/download/v1.0.0/fgp-huggingface-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/huggingface/releases/download/v1.0.0/fgp-huggingface-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/huggingface/releases/download/v1.0.0/fgp-huggingface-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-huggingface"
    (var/"fgp/services/huggingface").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        export HF_API_TOKEN="your-api-token"
        fgp start huggingface
        fgp call huggingface.inference --model "meta-llama/Llama-2-7b-chat-hf" --inputs "Hello, how are you?"
    EOS
  end

  service do
    run [opt_bin/"fgp-huggingface", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/huggingface"
    log_path var/"log/fgp-huggingface.log"
  end

  test do
    assert_match "fgp-huggingface", shell_output("#{bin}/fgp-huggingface --version")
  end
end
