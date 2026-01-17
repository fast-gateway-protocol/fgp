# typed: false
# frozen_string_literal: true

class FgpElasticsearch < Formula
  desc "FGP Elasticsearch daemon - Fast search and indexing operations"
  homepage "https://github.com/fast-gateway-protocol/elasticsearch"
  license "MIT"
  version "1.0.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/elasticsearch/releases/download/v1.0.0/fgp-elasticsearch-macos-arm64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/elasticsearch/releases/download/v1.0.0/fgp-elasticsearch-macos-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/fast-gateway-protocol/elasticsearch/releases/download/v1.0.0/fgp-elasticsearch-linux-x64.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-elasticsearch"
    (var/"fgp/services/elasticsearch").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        export ELASTICSEARCH_URL="http://localhost:9200"
        fgp start elasticsearch
        fgp call es.search --index products --query '{"match_all": {}}'
    EOS
  end

  service do
    run [opt_bin/"fgp-elasticsearch", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/elasticsearch"
    log_path var/"log/fgp-elasticsearch.log"
  end

  test do
    assert_match "fgp-elasticsearch", shell_output("#{bin}/fgp-elasticsearch --version")
  end
end
