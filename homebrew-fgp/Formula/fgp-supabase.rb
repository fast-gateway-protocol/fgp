# typed: false
# frozen_string_literal: true

class FgpSupabase < Formula
  desc "FGP Supabase daemon - Fast SQL, Auth, Storage, and Vectors"
  homepage "https://github.com/fast-gateway-protocol/supabase"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/supabase/releases/download/v#{version}/fgp-supabase-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/supabase/releases/download/v#{version}/fgp-supabase-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/supabase/releases/download/v#{version}/fgp-supabase-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-supabase"
    (var/"fgp/services/supabase").mkpath
  end

  def caveats
    <<~EOS
      Supabase daemon requires project credentials.

      Set environment variables:
        export SUPABASE_URL="https://xxx.supabase.co"
        export SUPABASE_KEY="your-anon-or-service-key"

      Quick start:
        fgp start supabase                       # Start daemon
        fgp call supabase sql "SELECT * FROM x"  # Run SQL
        fgp call supabase storage list           # List buckets

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/supabase/
    EOS
  end

  service do
    run [opt_bin/"fgp-supabase", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/supabase"
    log_path var/"log/fgp-supabase.log"
    error_log_path var/"log/fgp-supabase.log"
  end

  test do
    assert_match "fgp-supabase", shell_output("#{bin}/fgp-supabase --version")
  end
end
