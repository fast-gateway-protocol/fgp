# typed: false
# frozen_string_literal: true

class FgpSqlite < Formula
  desc "FGP SQLite daemon - Fast local database operations"
  homepage "https://github.com/fast-gateway-protocol/sqlite"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/sqlite/releases/download/v#{version}/fgp-sqlite-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/sqlite/releases/download/v#{version}/fgp-sqlite-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/sqlite/releases/download/v#{version}/fgp-sqlite-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-sqlite"
    (var/"fgp/services/sqlite").mkpath
  end

  def caveats
    <<~EOS
      SQLite daemon provides ultra-fast local database queries.

      Quick start:
        fgp start sqlite                            # Start daemon
        fgp call sqlite tables mydb.db              # List tables
        fgp call sqlite query mydb.db "SELECT..."   # Run query
        fgp call sqlite describe mydb.db users      # Describe table

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/sqlite/
    EOS
  end

  service do
    run [opt_bin/"fgp-sqlite", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/sqlite"
    log_path var/"log/fgp-sqlite.log"
    error_log_path var/"log/fgp-sqlite.log"
  end

  test do
    assert_match "fgp-sqlite", shell_output("#{bin}/fgp-sqlite --version")
  end
end
