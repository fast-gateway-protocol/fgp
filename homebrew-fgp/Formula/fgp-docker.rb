# typed: false
# frozen_string_literal: true

class FgpDocker < Formula
  desc "FGP Docker daemon - Fast container management via Docker API"
  homepage "https://github.com/fast-gateway-protocol/docker"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/docker/releases/download/v1.0.0/fgp-docker-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/docker/releases/download/v1.0.0/fgp-docker-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/docker/releases/download/v1.0.0/fgp-docker-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-docker"
    (var/"fgp/services/docker").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        fgp start docker
        fgp call docker.containers
        fgp call docker.logs --container my-app --tail 50
    EOS
  end

  service do
    run [opt_bin/"fgp-docker", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/docker"
    log_path var/"log/fgp-docker.log"
  end

  test do
    assert_match "fgp-docker", shell_output("#{bin}/fgp-docker --version")
  end
end
