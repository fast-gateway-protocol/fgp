# typed: false
# frozen_string_literal: true

class FgpContacts < Formula
  desc "FGP Contacts daemon - Fast macOS Contacts.framework access"
  homepage "https://github.com/fast-gateway-protocol/contacts"
  license "MIT"
  version "0.1.0"

  depends_on :macos

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/contacts/releases/download/v#{version}/fgp-contacts-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/contacts/releases/download/v#{version}/fgp-contacts-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-contacts"
    (var/"fgp/services/contacts").mkpath
  end

  def caveats
    <<~EOS
      Contacts daemon requires macOS Contacts permission.

      Grant access in System Preferences > Privacy & Security > Contacts.

      Quick start:
        fgp start contacts                  # Start daemon
        fgp call contacts search "John"     # Search contacts
        fgp call contacts get <id>          # Get contact details

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/contacts/
    EOS
  end

  service do
    run [opt_bin/"fgp-contacts", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/contacts"
    log_path var/"log/fgp-contacts.log"
    error_log_path var/"log/fgp-contacts.log"
  end

  test do
    assert_match "fgp-contacts", shell_output("#{bin}/fgp-contacts --version")
  end
end
