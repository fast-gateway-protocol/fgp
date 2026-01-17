# typed: false
# frozen_string_literal: true

class FgpTravel < Formula
  desc "FGP travel daemon - Flight & hotel search via Kiwi/Xotelo APIs (45-230x faster than MCP)"
  homepage "https://github.com/fast-gateway-protocol/travel"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/travel/releases/download/v#{version}/fgp-travel-macos-arm64.tar.gz"
      sha256 "244c5c505a182424e81b5673452c8d31f156cdfb91aa0e29508d1d534732c3b4"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/travel/releases/download/v#{version}/fgp-travel-macos-x64.tar.gz"
      sha256 "8f26ded6385e07578b2e0108c3a72e36f87459a2a5858a02f2561ab6237db4a1"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/fast-gateway-protocol/travel/releases/download/v#{version}/fgp-travel-linux-arm64.tar.gz"
      sha256 "e01b65120887befe3f45dd71b192ee39326ec25ed22e26db16122be7a9327d2e"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/travel/releases/download/v#{version}/fgp-travel-linux-x64.tar.gz"
      sha256 "d65a793389cbfeb521914f59d4463a947c96d493ffe292104d4c8b72f899b34f"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-travel"

    # Create service directory
    (var/"fgp/services/travel").mkpath
  end

  def post_install
    ohai "FGP Travel daemon installed!"
    ohai "Start with: fgp start travel"
  end

  def caveats
    <<~EOS
      The travel daemon provides fast flight and hotel search via Kiwi and Xotelo APIs.

      Required environment variable:
        export KIWI_API_KEY=your_api_key  # Get from https://tequila.kiwi.com/

      Quick start:
        fgp start travel                   # Start daemon

        # Location search
        fgp call travel find_location '{"term": "SFO"}'

        # Flight search
        fgp call travel search_flights '{"origin": "SFO", "destination": "LAX", "date": "2025-03-15"}'

        # Token-efficient methods (designed for LLM agents)
        fgp call travel price_check '{"origin": "SFO", "destination": "LAX", "date": "2025-03-15"}'
        fgp call travel search_cheapest_day '{"origin": "SFO", "destination": "LAX", "month": "2025-03"}'

      Performance: 10-50ms per call (vs 2,300ms MCP stdio) - 45-230x faster!

      Documentation: https://github.com/fast-gateway-protocol/travel
    EOS
  end

  service do
    run [opt_bin/"fgp-travel", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/travel"
    log_path var/"log/fgp-travel.log"
    error_log_path var/"log/fgp-travel.log"
  end

  test do
    assert_match "fgp-travel", shell_output("#{bin}/fgp-travel --version")
  end
end
