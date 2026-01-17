# typed: false
# frozen_string_literal: true

class FgpOllama < Formula
  desc "FGP Ollama daemon - Fast local LLM inference via Ollama"
  homepage "https://github.com/fast-gateway-protocol/ollama"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/ollama/releases/download/v1.0.0/fgp-ollama-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/ollama/releases/download/v1.0.0/fgp-ollama-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/ollama/releases/download/v1.0.0/fgp-ollama-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-ollama"
    (var/"fgp/services/ollama").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        # OLLAMA_HOST is optional, defaults to http://localhost:11434
        # export OLLAMA_HOST="http://localhost:11434"
        fgp start ollama
        fgp call ollama.chat --model "llama2" --messages '[{"role": "user", "content": "Hello"}]'

      Note: Requires Ollama to be running locally. Install with:
        brew install ollama
        ollama serve
    EOS
  end

  service do
    run [opt_bin/"fgp-ollama", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/ollama"
    log_path var/"log/fgp-ollama.log"
  end

  test do
    assert_match "fgp-ollama", shell_output("#{bin}/fgp-ollama --version")
  end
end
