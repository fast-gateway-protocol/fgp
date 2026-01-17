# typed: false
# frozen_string_literal: true

class FgpGoogleDocs < Formula
  desc "FGP Google Docs daemon - Fast document operations via Google API"
  homepage "https://github.com/fast-gateway-protocol/google-docs"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/google-docs/releases/download/v#{version}/fgp-google-docs-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/google-docs/releases/download/v#{version}/fgp-google-docs-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/google-docs/releases/download/v#{version}/fgp-google-docs-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-google-docs"
    (var/"fgp/services/google-docs").mkpath
  end

  def caveats
    <<~EOS
      Google Docs daemon requires OAuth setup on first use.

      Quick start:
        fgp start google-docs               # Start daemon
        fgp call google-docs auth           # Authenticate
        fgp call google-docs read <doc-id>  # Read document

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/google-docs/
    EOS
  end

  service do
    run [opt_bin/"fgp-google-docs", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/google-docs"
    log_path var/"log/fgp-google-docs.log"
    error_log_path var/"log/fgp-google-docs.log"
  end

  test do
    assert_match "fgp-google-docs", shell_output("#{bin}/fgp-google-docs --version")
  end
end
