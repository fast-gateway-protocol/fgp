# FGP Benchmarks

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

| Operation | MCP Stdio | FGP Daemon | Speedup |
|-----------|-----------|------------|---------|
| Browser navigate | 2,300ms | 8ms | **292x** |
| Browser snapshot | 2,300ms | 9ms | **257x** |
| Gmail list | 2,400ms | 35ms | **69x** |
| GitHub issues | 2,100ms | 28ms | **75x** |

## Output

Benchmarks output:
- Console: Colored table with timing stats
- JSON: Structured results for analysis (use `--json` flag where supported)

## Notes

- Browser benchmarks require Chrome running with remote debugging
- Gmail/Calendar benchmarks require OAuth credentials in `~/.fgp/auth/google/`
- GitHub benchmarks require `GITHUB_TOKEN` environment variable
