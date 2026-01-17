# typed: false
# frozen_string_literal: true

class FgpPandoc < Formula
  desc "FGP Pandoc daemon - Fast document conversion"
  homepage "https://github.com/fast-gateway-protocol/pandoc"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/pandoc/releases/download/v#{version}/fgp-pandoc-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/pandoc/releases/download/v#{version}/fgp-pandoc-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/pandoc/releases/download/v#{version}/fgp-pandoc-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"
  depends_on "pandoc"

  def install
    bin.install "fgp-pandoc"
    (var/"fgp/services/pandoc").mkpath
  end

  def caveats
    <<~EOS
      Pandoc daemon wraps your system Pandoc for fast document conversion.

      For PDF output, install LaTeX:
        brew install basictex

      Quick start:
        fgp start pandoc                           # Start daemon
        fgp call pandoc convert doc.md --to pdf    # Markdown to PDF
        fgp call pandoc convert doc.md --to docx   # Markdown to Word
        fgp call pandoc convert doc.docx --to md   # Word to Markdown

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/pandoc/
    EOS
  end

  service do
    run [opt_bin/"fgp-pandoc", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/pandoc"
    log_path var/"log/fgp-pandoc.log"
    error_log_path var/"log/fgp-pandoc.log"
  end

  test do
    assert_match "fgp-pandoc", shell_output("#{bin}/fgp-pandoc --version")
  end
end
