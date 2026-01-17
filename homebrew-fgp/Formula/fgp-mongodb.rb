# typed: false
# frozen_string_literal: true

class FgpMongodb < Formula
  desc "FGP MongoDB daemon - Fast MongoDB operations with connection pooling"
  homepage "https://github.com/fast-gateway-protocol/mongodb"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/mongodb/releases/download/v1.0.0/fgp-mongodb-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/mongodb/releases/download/v1.0.0/fgp-mongodb-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/mongodb/releases/download/v1.0.0/fgp-mongodb-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-mongodb"
    (var/"fgp/services/mongodb").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        export MONGODB_URI="mongodb://localhost:27017/mydb"
        fgp start mongodb
        fgp call mongodb.find --collection users
    EOS
  end

  service do
    run [opt_bin/"fgp-mongodb", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/mongodb"
    log_path var/"log/fgp-mongodb.log"
  end

  test do
    assert_match "fgp-mongodb", shell_output("#{bin}/fgp-mongodb --version")
  end
end
