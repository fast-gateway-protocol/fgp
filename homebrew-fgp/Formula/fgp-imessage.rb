# typed: false
# frozen_string_literal: true

class FgpImessage < Formula
  desc "FGP iMessage daemon - Fast iMessage access via SQLite"
  homepage "https://github.com/fast-gateway-protocol/imessage"
  license "MIT"
  version "0.1.0"

  depends_on :macos

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/imessage/releases/download/v#{version}/fgp-imessage-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/imessage/releases/download/v#{version}/fgp-imessage-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-imessage"
    (var/"fgp/services/imessage").mkpath
  end

  def caveats
    <<~EOS
      iMessage daemon requires Full Disk Access for chat.db.

      Grant access in System Preferences > Privacy & Security > Full Disk Access.

      Quick start:
        fgp start imessage                   # Start daemon
        fgp call imessage recent             # Recent messages
        fgp call imessage send "Mom" "Hi!"   # Send message

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/imessage/
    EOS
  end

  service do
    run [opt_bin/"fgp-imessage", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/imessage"
    log_path var/"log/fgp-imessage.log"
    error_log_path var/"log/fgp-imessage.log"
  end

  test do
    assert_match "fgp-imessage", shell_output("#{bin}/fgp-imessage --version")
  end
end
