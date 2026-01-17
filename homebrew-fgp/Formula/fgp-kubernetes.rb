# typed: false
# frozen_string_literal: true

class FgpKubernetes < Formula
  desc "FGP Kubernetes daemon - Fast cluster operations via K8s API"
  homepage "https://github.com/fast-gateway-protocol/kubernetes"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/kubernetes/releases/download/v1.0.0/fgp-kubernetes-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/kubernetes/releases/download/v1.0.0/fgp-kubernetes-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/kubernetes/releases/download/v1.0.0/fgp-kubernetes-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-kubernetes"
    (var/"fgp/services/kubernetes").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        fgp start kubernetes
        fgp call k8s.pods --namespace default
        fgp call k8s.logs --pod my-app --tail 100
    EOS
  end

  service do
    run [opt_bin/"fgp-kubernetes", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/kubernetes"
    log_path var/"log/fgp-kubernetes.log"
  end

  test do
    assert_match "fgp-kubernetes", shell_output("#{bin}/fgp-kubernetes --version")
  end
end
