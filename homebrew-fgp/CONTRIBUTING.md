# Contributing to homebrew-fgp

## Adding a New Daemon Formula

When you add a new FGP daemon, follow these steps to add it to Homebrew:

### 1. Ensure the daemon has a release

The daemon must have a GitHub release with pre-built binaries for:
- `fgp-{name}-macos-arm64.tar.gz`
- `fgp-{name}-macos-x64.tar.gz`
- `fgp-{name}-linux-arm64.tar.gz`
- `fgp-{name}-linux-x64.tar.gz`

### 2. Get SHA256 hashes

Run the update script or manually fetch:

```bash
# Using the update script
./scripts/update-shas.sh 0.1.0

# Or manually
curl -sL "https://github.com/fast-gateway-protocol/{daemon}/releases/download/v0.1.0/fgp-{daemon}-macos-arm64.tar.gz" | shasum -a 256
```

### 3. Create the formula

Copy an existing formula and modify:

```ruby
# Formula/fgp-{daemon}.rb
class Fgp{Daemon} < Formula
  desc "FGP {Daemon} daemon - {description}"
  homepage "https://github.com/fast-gateway-protocol/{daemon}"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/fast-gateway-protocol/{daemon}/releases/download/v#{version}/fgp-{daemon}-macos-arm64.tar.gz"
      sha256 "{sha256}"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/{daemon}/releases/download/v#{version}/fgp-{daemon}-macos-x64.tar.gz"
      sha256 "{sha256}"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/fast-gateway-protocol/{daemon}/releases/download/v#{version}/fgp-{daemon}-linux-arm64.tar.gz"
      sha256 "{sha256}"
    end
    on_intel do
      url "https://github.com/fast-gateway-protocol/{daemon}/releases/download/v#{version}/fgp-{daemon}-linux-x64.tar.gz"
      sha256 "{sha256}"
    end
  end

  depends_on "fgp"

  def install
    bin.install "fgp-{daemon}"
    (var/"fgp/services/{daemon}").mkpath
  end

  def caveats
    <<~EOS
      Quick start:
        fgp start {daemon}
        fgp call {daemon} {method}

      Documentation: https://fast-gateway-protocol.github.io/fgp/daemons/{daemon}/
    EOS
  end

  service do
    run [opt_bin/"fgp-{daemon}", "start", "--foreground"]
    keep_alive true
    working_dir var/"fgp/services/{daemon}"
    log_path var/"log/fgp-{daemon}.log"
    error_log_path var/"log/fgp-{daemon}.log"
  end

  test do
    assert_match "fgp-{daemon}", shell_output("#{bin}/fgp-{daemon} --version")
  end
end
```

### 4. Test locally

```bash
brew install --build-from-source ./Formula/fgp-{daemon}.rb
fgp-{daemon} --version
```

### 5. Update README

Add the new formula to the table in README.md.

### 6. Commit and push

```bash
git add Formula/fgp-{daemon}.rb README.md
git commit -m "feat: add fgp-{daemon} formula"
git push
```

## Updating Version

When releasing a new version:

1. Update `version` in each formula
2. Get new SHA256 hashes: `./scripts/update-shas.sh {version}`
3. Update all formulas with new hashes
4. Commit: `git commit -am "chore: bump to v{version}"`
5. Push: `git push`

## Checklist for New Daemons

- [ ] Daemon has GitHub release with binaries
- [ ] SHA256 hashes computed for all 4 platforms
- [ ] Formula created in `Formula/fgp-{daemon}.rb`
- [ ] Formula tested locally with `brew install --build-from-source`
- [ ] README.md updated with new formula
- [ ] Changes committed and pushed
