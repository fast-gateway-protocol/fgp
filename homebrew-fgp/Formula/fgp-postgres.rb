# typed: false
# frozen_string_literal: true

class FgpPostgres < Formula
  desc "FGP Postgres daemon - Direct PostgreSQL operations with connection pooling"
  homepage "https://github.com/fast-gateway-protocol/postgres"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/postgres/releases/download/v#{version}/fgp-postgres-macos-arm64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_ARM64"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/postgres/releases/download/v#{version}/fgp-postgres-macos-x64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_X64"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/fast-gateway-protocol/postgres/releases/download/v#{version}/fgp-postgres-linux-arm64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/postgres/releases/download/v#{version}/fgp-postgres-linux-x64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X64"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-postgres"
    (var/"fgp/services/postgres").mkpath
  end

  def caveats
    <<~EOS
      Postgres daemon requires a DATABASE_URL.
      Set DATABASE_URL or PGHOST/PGUSER/etc environment variables.

      Quick start:
        fgp start postgres                   # Start daemon
        fgp call postgres tables             # List tables
        fgp call postgres query "SELECT 1"   # Run SQL

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/postgres/
    EOS
  end

  service do
    run [opt_bin/"fgp-postgres", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/postgres"
    log_path var/"log/fgp-postgres.log"
    error_log_path var/"log/fgp-postgres.log"
  end

  test do
    assert_match "fgp-postgres", shell_output("#{bin}/fgp-postgres --version")
  end
end
