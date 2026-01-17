# typed: false
# frozen_string_literal: true

class FgpAirtable < Formula
  desc "FGP Airtable daemon - Fast database operations"
  homepage "https://github.com/fast-gateway-protocol/airtable"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/airtable/releases/download/v1.0.0/fgp-airtable-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/airtable/releases/download/v1.0.0/fgp-airtable-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/airtable/releases/download/v1.0.0/fgp-airtable-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-airtable"
    (var/"fgp/services/airtable").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        export AIRTABLE_PAT="pat..."
        fgp start airtable
        fgp call airtable.records --base appXXX --table "Tasks"
    EOS
  end

  service do
    run [opt_bin/"fgp-airtable", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/airtable"
    log_path var/"log/fgp-airtable.log"
  end

  test do
    assert_match "fgp-airtable", shell_output("#{bin}/fgp-airtable --version")
  end
end
