# typed: false
# frozen_string_literal: true

class FgpImagemagick < Formula
  desc "FGP ImageMagick daemon - Fast image processing"
  homepage "https://github.com/fast-gateway-protocol/imagemagick"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/imagemagick/releases/download/v#{version}/fgp-imagemagick-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/imagemagick/releases/download/v#{version}/fgp-imagemagick-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/imagemagick/releases/download/v#{version}/fgp-imagemagick-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"
  depends_on "imagemagick"

  def install
    bin.install "fgp-imagemagick"
    (var/"fgp/services/imagemagick").mkpath
  end

  def caveats
    <<~EOS
      ImageMagick daemon wraps your system ImageMagick for fast image processing.

      Quick start:
        fgp start imagemagick                        # Start daemon
        fgp call im info image.jpg                   # Get image info
        fgp call im resize image.jpg --width 800     # Resize image
        fgp call im convert image.png --to webp      # Convert format

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/imagemagick/
    EOS
  end

  service do
    run [opt_bin/"fgp-imagemagick", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/imagemagick"
    log_path var/"log/fgp-imagemagick.log"
    error_log_path var/"log/fgp-imagemagick.log"
  end

  test do
    assert_match "fgp-imagemagick", shell_output("#{bin}/fgp-imagemagick --version")
  end
end
