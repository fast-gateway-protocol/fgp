# typed: false
# frozen_string_literal: true

class FgpSlack < Formula
  desc "FGP Slack daemon - Fast channel and message operations"
  homepage "https://github.com/fast-gateway-protocol/slack"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/slack/releases/download/v#{version}/fgp-slack-macos-arm64.tar.gz"
      sha256 "b63ea2a871df8d584bc47db30d9d2dfdad9ee9aa0cd879bc68190623a47a6a62"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/slack/releases/download/v#{version}/fgp-slack-macos-x64.tar.gz"
      sha256 "530ade2ae621db5573e62f5d12e8da72f0ab8be4563eb173ce76e7a098e7af3c"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/fast-gateway-protocol/slack/releases/download/v#{version}/fgp-slack-linux-arm64.tar.gz"
      sha256 "723cbaf8faab743ab7b957d25b4e2cefbeafd2e331f80ba0ea22bb8876179d93"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/slack/releases/download/v#{version}/fgp-slack-linux-x64.tar.gz"
      sha256 "b9fefa524b3315af1c8d5bcf70d6bce023a54e340d367fee20785326843bc13e"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-slack"
    (var/"fgp/services/slack").mkpath
  end

  def caveats
    <<~EOS
      Slack daemon requires a Slack Bot token.
      Set SLACK_TOKEN environment variable or use --token flag.

      Create a Slack app at https://api.slack.com/apps with scopes:
        channels:read, channels:history, chat:write, users:read

      Quick start:
        export SLACK_TOKEN="xoxb-your-token"
        fgp start slack                    # Start daemon
        fgp call slack channels            # List channels
        fgp call slack send '{"channel": "C01234", "text": "Hello!"}'

      Documentation: https://fast-gateway-protocol.github.io/fgp/
    EOS
  end

  service do
    run [opt_bin/"fgp-slack", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/slack"
    log_path var/"log/fgp-slack.log"
    error_log_path var/"log/fgp-slack.log"
  end

  test do
    assert_match "fgp-slack", shell_output("#{bin}/fgp-slack --version")
  end
end
