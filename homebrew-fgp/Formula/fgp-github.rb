# typed: false
# frozen_string_literal: true

class FgpGithub < Formula
  desc "FGP GitHub daemon - Fast GitHub operations via GraphQL + REST"
  homepage "https://github.com/fast-gateway-protocol/github"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/github/releases/download/v#{version}/fgp-github-macos-arm64.tar.gz"
      sha256 "5219f8e5514af6c7f1ad0d4f424b3ed35bbc80d076910b9049cb4da4948fd026"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/github/releases/download/v#{version}/fgp-github-macos-x64.tar.gz"
      sha256 "bba28b059920d35c837498510d6cb8f24196ba7683f482bfcabe37da7cb9b7b4"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/fast-gateway-protocol/github/releases/download/v#{version}/fgp-github-linux-arm64.tar.gz"
      sha256 "d5aaad6490a107c1396a8d08eef9e5e5a6d31d5fc5ce6604b35eb743cf34cfdf"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/github/releases/download/v#{version}/fgp-github-linux-x64.tar.gz"
      sha256 "41e9d9ae738afad371d3aedc8b7c37a90b6a315161a9df563333df534e8b7263"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-github"
    (var/"fgp/services/github").mkpath
  end

  def caveats
    <<~EOS
      GitHub daemon uses `gh` CLI authentication.
      Run `gh auth login` if not already authenticated.

      Quick start:
        fgp start github                       # Start daemon
        fgp call github repos                  # List your repos
        fgp call github issues owner/repo      # List issues
        fgp call github notifications          # Get notifications

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/github/
    EOS
  end

  service do
    run [opt_bin/"fgp-github", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/github"
    log_path var/"log/fgp-github.log"
    error_log_path var/"log/fgp-github.log"
  end

  test do
    assert_match "fgp-github", shell_output("#{bin}/fgp-github --version")
  end
end
