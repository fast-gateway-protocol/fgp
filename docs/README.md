# Documentation Site

This directory contains the MkDocs source for the Fast Gateway Protocol documentation at https://fast-gateway-protocol.github.io/fgp/.

## Structure

- `index.md` - Landing page
- `getting-started/` - Installation and quickstart guides
- `daemons/` - Service-specific docs
- `protocol/` - NDJSON protocol spec
- `reference/` - CLI + API reference
- `development/` - Contributor and build docs
- `assets/` - Images and charts used in docs

## Local development

```bash
cd docs
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
mkdocs serve
```

## Build

```bash
mkdocs build
```

## Updating benchmarks

Benchmark charts are generated from `benchmarks/generate_charts.py` in the repo root and written to `docs/assets/`.
