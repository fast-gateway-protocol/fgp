# FGP Benchmarks

## Doctrine

See [DOCTRINE.md](./DOCTRINE.md).


Performance benchmarks comparing FGP daemons against MCP servers and other automation tools.

## Benchmark Files

| File | Description | Measures |
|------|-------------|----------|
| `browser_benchmark.py` | Browser daemon core operations | Navigate, snapshot, click latency |
| `browser_workflow_benchmark.py` | Multi-step browser workflows | End-to-end workflow timing |
| `daemon_benchmarks.py` | FGP SDK latency tests | Socket connection, dispatch overhead |
| `fgp_api_benchmark.py` | API surface testing | Method coverage, response times |
| `fgp_vs_mcp_benchmark.py` | Head-to-head comparison | FGP vs MCP stdio servers |
| `gmail_workflow_benchmark.py` | Gmail operations | List, read, send timing |
| `hn_workflow_benchmark.py` | HackerNews demo workflow | Real-world scraping scenario |
| `mcp_server_benchmarks.py` | MCP baseline measurements | Cold-start overhead baseline |

## Requirements

```bash
pip install rich tabulate statistics
```

## Quick Start

```bash
# Run browser benchmark (5 iterations)
python3 browser_benchmark.py --iterations 5

# Run FGP vs MCP comparison
python3 fgp_vs_mcp_benchmark.py --iterations 10

# Run all benchmarks
for f in *.py; do python3 "$f" --iterations 3; done
```

## Key Results

> **Methodology Note:** These benchmarks compare **warm-to-warm** performance (both FGP daemon and MCP server already running). Cold start elimination is measured separately.

### Cold Start Elimination

MCP servers spawn a new process per session. FGP daemons stay warm.

| Service | MCP Cold Start | FGP | Savings |
|---------|----------------|-----|---------|
| Browser (Playwright) | 1,080ms | 0ms | **1.1s saved** |
| Gmail | 1,597ms | 0ms | **1.6s saved** |
| GitHub | ~1,000ms | 0ms | **1.0s saved** |

### Warm-to-Warm Comparison

Once both are running, network-bound operations are comparable:

| Operation | MCP Warm | FGP Warm | Speedup |
|-----------|----------|----------|---------|
| Gmail unread count | 145ms | 142ms | ~1x |
| Gmail inbox (10 msgs) | 323ms | 333ms | ~1x |
| GitHub user | 300ms | 310ms | ~1x |
| Browser snapshot | 2.6ms | 3.2ms | ~1x |

### Local Operations (No Network)

FGP shines for local database/file operations:

| Operation | CLI/Subprocess | FGP Daemon | Speedup |
|-----------|----------------|------------|---------|
| Screen Time daily | 5.0ms | 0.32ms | **15.7x** |
| iMessage recent | 80ms | 5ms | **16x** |
| iMessage analytics | 100ms | 5ms | **20x** |

### Summary

- **Network APIs (Gmail, GitHub):** ~1x - network latency dominates
- **Local SQLite (Screen Time, iMessage):** 14-20x faster
- **Cold start:** 1.0-1.6s eliminated per session

## Output

Benchmarks output:
- Console: Colored table with timing stats
- JSON: Structured results for analysis (use `--json` flag where supported)

## Notes

- Browser benchmarks require Chrome running with remote debugging
- Gmail/Calendar benchmarks require OAuth credentials in `~/.fgp/auth/google/`
- GitHub benchmarks require `GITHUB_TOKEN` environment variable
- iMessage benchmarks require macOS with Full Disk Access permission
