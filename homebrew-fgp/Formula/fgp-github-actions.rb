# typed: false
# frozen_string_literal: true

class FgpGithubActions < Formula
  desc "FGP GitHub Actions daemon - Fast workflow management"
  homepage "https://github.com/fast-gateway-protocol/github-actions"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/github-actions/releases/download/v1.0.0/fgp-github-actions-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/github-actions/releases/download/v1.0.0/fgp-github-actions-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/github-actions/releases/download/v1.0.0/fgp-github-actions-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-github-actions"
    (var/"fgp/services/github-actions").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        export GITHUB_TOKEN="ghp_..."
        fgp start github-actions
        fgp call actions.runs --repo owner/repo --limit 10
    EOS
  end

  service do
    run [opt_bin/"fgp-github-actions", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/github-actions"
    log_path var/"log/fgp-github-actions.log"
  end

  test do
    assert_match "fgp-github-actions", shell_output("#{bin}/fgp-github-actions --version")
  end
end
