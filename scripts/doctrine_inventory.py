#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List

DOCTRINE_CANDIDATES = [
    "DOCTRINE.md",
    "Doctrine.md",
    "doctrine.md",
    "docs/DOCTRINE.md",
    "docs/Doctrine.md",
    "docs/doctrine.md",
    "docs/doctrines.md",
    "docs/Doctrines.md",
    "docs/doctrines/README.md",
    "docs/doctrine/README.md",
]

MODULE_MARKERS = [
    "README.md",
    "Cargo.toml",
    "pyproject.toml",
    "package.json",
    "go.mod",
    "requirements.txt",
    "src",
]

LANG_MARKERS = {
    "rust": ["Cargo.toml"],
    "python": ["pyproject.toml", "requirements.txt"],
    "node": ["package.json"],
    "go": ["go.mod"],
}

DEFAULT_REQUIRED_HEADINGS = [
    "Purpose",
    "Scope",
    "Non-Goals",
    "Tenets",
    "Architecture",
    "Interfaces",
    "Operational Model",
    "Testing",
    "Security",
    "Observability",
    "Risks",
    "Roadmap",
]

DEFAULT_EXCLUDES = {
    "Plans",
    "docs",
    "logs",
    "notes",
}


@dataclass(frozen=True)
class ModuleRecord:
    name: str
    path: str
    languages: List[str]
    has_readme: bool
    has_docs_dir: bool
    has_src_dir: bool
    doctrine_files: List[str]
    headings: List[str]
    missing_headings: List[str]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Inventory doctrine coverage across top-level modules."
    )
    parser.add_argument(
        "--root",
        default=None,
        help="Repository root (defaults to script parent directory).",
    )
    parser.add_argument(
        "--format",
        choices=["markdown", "json", "csv", "tsv"],
        default="markdown",
        help="Output format.",
    )
    parser.add_argument(
        "--out",
        default=None,
        help="Write output to a file instead of stdout.",
    )
    parser.add_argument(
        "--required-heading",
        action="append",
        dest="required_headings",
        help="Heading required in doctrine files. Can be repeated.",
    )
    parser.add_argument(
        "--exclude",
        action="append",
        dest="excludes",
        help="Top-level directory name to exclude. Can be repeated.",
    )
    parser.add_argument(
        "--no-default-excludes",
        action="store_true",
        help="Do not exclude default non-module directories.",
    )
    parser.add_argument(
        "--no-default-headings",
        action="store_true",
        help="Do not use default required headings.",
    )
    return parser.parse_args()


def repo_root_from_script() -> Path:
    return Path(__file__).resolve().parents[1]


def is_module_dir(path: Path) -> bool:
    if not path.is_dir():
        return False
    if path.name.startswith("."):
        return False
    for marker in MODULE_MARKERS:
        if (path / marker).exists():
            return True
    return False


def detect_languages(path: Path) -> List[str]:
    languages = []
    for lang, markers in LANG_MARKERS.items():
        if any((path / marker).exists() for marker in markers):
            languages.append(lang)
    return languages


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except Exception:
        return path.read_text(encoding="utf-8", errors="ignore")


def extract_headings(text: str) -> List[str]:
    headings = []
    for line in text.splitlines():
        line = line.strip()
        if line.startswith("#"):
            parts = line.lstrip("#").strip()
            if parts:
                headings.append(parts)
    return headings


def find_doctrine_files(module_path: Path) -> List[Path]:
    found_map = {}

    def add_path(path: Path) -> None:
        try:
            key = str(path.resolve()).lower()
        except FileNotFoundError:
            key = str(path).lower()
        if key not in found_map:
            found_map[key] = path

    for rel in DOCTRINE_CANDIDATES:
        candidate = module_path / rel
        if candidate.exists():
            add_path(candidate)

    docs_dir = module_path / "docs"
    if docs_dir.exists():
        for path in docs_dir.rglob("*doctrine*.md"):
            add_path(path)

    root_matches = list(module_path.glob("*doctrine*.md"))
    for path in root_matches:
        add_path(path)

    return sorted(found_map.values())


def normalize_heading(name: str) -> str:
    return " ".join(name.split()).strip().lower()


def build_record(module_path: Path, required_headings: Iterable[str]) -> ModuleRecord:
    doctrine_files = find_doctrine_files(module_path)
    headings: List[str] = []
    for doc in doctrine_files:
        headings.extend(extract_headings(read_text(doc)))

    normalized = {normalize_heading(h) for h in headings}
    required = [h for h in required_headings if h]
    missing = []
    for heading in required:
        if normalize_heading(heading) not in normalized:
            missing.append(heading)

    return ModuleRecord(
        name=module_path.name,
        path=str(module_path),
        languages=detect_languages(module_path),
        has_readme=(module_path / "README.md").exists(),
        has_docs_dir=(module_path / "docs").exists(),
        has_src_dir=(module_path / "src").exists(),
        doctrine_files=[str(p.relative_to(module_path)) for p in doctrine_files],
        headings=sorted(set(headings)),
        missing_headings=missing,
    )


def to_markdown(rows: List[ModuleRecord], include_headings: bool) -> str:
    header = [
        "Module",
        "Lang",
        "README",
        "Docs",
        "Src",
        "Doctrine Files",
    ]
    if include_headings:
        header.append("Missing Headings")

    lines = ["| " + " | ".join(header) + " |", "|" + "|".join([" --- "] * len(header)) + "|"]

    for row in rows:
        doctrine_files = ", ".join(row.doctrine_files) if row.doctrine_files else "-"
        missing = ", ".join(row.missing_headings) if row.missing_headings else "-"
        langs = ", ".join(row.languages) if row.languages else "-"
        cells = [
            row.name,
            langs,
            "yes" if row.has_readme else "no",
            "yes" if row.has_docs_dir else "no",
            "yes" if row.has_src_dir else "no",
            doctrine_files,
        ]
        if include_headings:
            cells.append(missing)
        lines.append("| " + " | ".join(cells) + " |")

    return "\n".join(lines) + "\n"


def write_csv(rows: List[ModuleRecord], out, delimiter: str) -> None:
    fieldnames = [
        "module",
        "path",
        "languages",
        "has_readme",
        "has_docs_dir",
        "has_src_dir",
        "doctrine_files",
        "headings",
        "missing_headings",
    ]
    writer = csv.DictWriter(out, fieldnames=fieldnames, delimiter=delimiter)
    writer.writeheader()
    for row in rows:
        writer.writerow(
            {
                "module": row.name,
                "path": row.path,
                "languages": ",".join(row.languages),
                "has_readme": row.has_readme,
                "has_docs_dir": row.has_docs_dir,
                "has_src_dir": row.has_src_dir,
                "doctrine_files": ",".join(row.doctrine_files),
                "headings": ",".join(row.headings),
                "missing_headings": ",".join(row.missing_headings),
            }
        )


def main() -> int:
    args = parse_args()
    root = Path(args.root).resolve() if args.root else repo_root_from_script()

    excludes = set(args.excludes or [])
    if not args.no_default_excludes:
        excludes |= DEFAULT_EXCLUDES

    required_headings: List[str] = []
    if not args.no_default_headings:
        required_headings.extend(DEFAULT_REQUIRED_HEADINGS)
    if args.required_headings:
        required_headings.extend(args.required_headings)

    modules = [
        path for path in root.iterdir() if path.name not in excludes and is_module_dir(path)
    ]
    records = [
        build_record(path, required_headings) for path in sorted(modules, key=lambda p: p.name)
    ]

    output = ""
    if args.format == "markdown":
        output = to_markdown(records, include_headings=bool(required_headings))
    elif args.format == "json":
        output = json.dumps([record.__dict__ for record in records], indent=2)
    elif args.format in {"csv", "tsv"}:
        delimiter = "," if args.format == "csv" else "\t"
        stream = open(args.out, "w", newline="", encoding="utf-8") if args.out else sys.stdout
        with stream:
            write_csv(records, stream, delimiter=delimiter)
        return 0

    if args.out:
        Path(args.out).write_text(output, encoding="utf-8")
    else:
        print(output)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
