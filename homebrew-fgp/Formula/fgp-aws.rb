# typed: false
# frozen_string_literal: true

class FgpAws < Formula
  desc "FGP AWS daemon - Fast S3, Lambda, DynamoDB operations"
  homepage "https://github.com/fast-gateway-protocol/aws"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/aws/releases/download/v1.0.0/fgp-aws-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/aws/releases/download/v1.0.0/fgp-aws-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/aws/releases/download/v1.0.0/fgp-aws-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-aws"
    (var/"fgp/services/aws").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        aws configure  # Or set AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY
        fgp start aws
        fgp call aws.s3.list
    EOS
  end

  service do
    run [opt_bin/"fgp-aws", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/aws"
    log_path var/"log/fgp-aws.log"
  end

  test do
    assert_match "fgp-aws", shell_output("#{bin}/fgp-aws --version")
  end
end
