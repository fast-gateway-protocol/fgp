# typed: false
# frozen_string_literal: true

class FgpFly < Formula
  desc "FGP Fly daemon - Fast Fly.io deployment operations"
  homepage "https://github.com/fast-gateway-protocol/fly"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/fly/releases/download/v#{version}/fgp-fly-macos-arm64.tar.gz"
      sha256 "cbf40063401d8d81f8bc32fbc793755eda0f2942b41149ec6eed145e1475798e"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/fly/releases/download/v#{version}/fgp-fly-macos-x64.tar.gz"
      sha256 "38109873041382b3c44f40f951aa113b27afef2ebcdb537d450082a8b6b0c310"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/fast-gateway-protocol/fly/releases/download/v#{version}/fgp-fly-linux-arm64.tar.gz"
      sha256 "5d058c31649db2329c52d1d657d349384623fb8d74adcb75c648243d21660bd6"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/fly/releases/download/v#{version}/fgp-fly-linux-x64.tar.gz"
      sha256 "339e41cd88dcbec7db7a491f1664f96fdecbd977bcf21115dcaa4f3c86937855"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-fly"
    (var/"fgp/services/fly").mkpath
  end

  def caveats
    <<~EOS
      Fly daemon uses `flyctl` authentication.
      Run `fly auth login` if not already authenticated.

      Quick start:
        fgp start fly                  # Start daemon
        fgp call fly apps              # List apps
        fgp call fly status app-name   # App status

      Documentation: https://fast-gateway-protocol.github.io/fgp/
    EOS
  end

  service do
    run [opt_bin/"fgp-fly", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/fly"
    log_path var/"log/fgp-fly.log"
    error_log_path var/"log/fgp-fly.log"
  end

  test do
    assert_match "fgp-fly", shell_output("#{bin}/fgp-fly --version")
  end
end
