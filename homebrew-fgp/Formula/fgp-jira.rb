# typed: false
# frozen_string_literal: true

class FgpJira < Formula
  desc "FGP Jira daemon - Fast issue tracking operations"
  homepage "https://github.com/fast-gateway-protocol/jira"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/jira/releases/download/v1.0.0/fgp-jira-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/jira/releases/download/v1.0.0/fgp-jira-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/jira/releases/download/v1.0.0/fgp-jira-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-jira"
    (var/"fgp/services/jira").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        export JIRA_URL="https://your-domain.atlassian.net"
        export JIRA_EMAIL="your@email.com"
        export JIRA_API_TOKEN="..."
        fgp start jira
        fgp call jira.search --jql "project = PROJ"
    EOS
  end

  service do
    run [opt_bin/"fgp-jira", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/jira"
    log_path var/"log/fgp-jira.log"
  end

  test do
    assert_match "fgp-jira", shell_output("#{bin}/fgp-jira --version")
  end
end
