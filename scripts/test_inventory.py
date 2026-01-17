#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List, Sequence

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

DEFAULT_EXCLUDES = {
    ".git",
    "Plans",
    "docs",
    "logs",
    "notes",
    "target",
    "node_modules",
    "dist",
    "build",
}

PYTHON_TEST_GLOBS = ["test_*.py", "*_test.py"]
NODE_TEST_GLOBS = ["*.test.js", "*.test.jsx", "*.test.ts", "*.test.tsx", "*.spec.js", "*.spec.jsx", "*.spec.ts", "*.spec.tsx"]
GO_TEST_GLOBS = ["*_test.go"]

RUST_INLINE_MARKERS = ["#[test]", "#[tokio::test]", "#[cfg(test)]", "mod tests"]


@dataclass(frozen=True)
class ModuleRecord:
    name: str
    path: str
    languages: List[str]
    has_readme: bool
    has_tests_dir: bool
    test_files: List[str]
    rust_inline_tests: int


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Inventory unit test coverage across top-level modules."
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
        "--include-files",
        action="store_true",
        help="Include test file list in markdown output.",
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


def is_excluded(path: Path, excludes: Iterable[str]) -> bool:
    return any(part in excludes for part in path.parts)


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except Exception:
        return path.read_text(encoding="utf-8", errors="ignore")


def glob_files(root: Path, patterns: Sequence[str], excludes: Iterable[str]) -> List[Path]:
    found: List[Path] = []
    for pattern in patterns:
        for path in root.rglob(pattern):
            if is_excluded(path, excludes):
                continue
            if path.is_file():
                found.append(path)
    return found


def rust_inline_test_count(module_path: Path, excludes: Iterable[str]) -> int:
    src_dir = module_path / "src"
    if not src_dir.exists():
        return 0
    count = 0
    for path in src_dir.rglob("*.rs"):
        if is_excluded(path, excludes):
            continue
        content = read_text(path)
        if any(marker in content for marker in RUST_INLINE_MARKERS):
            count += 1
    return count


def find_test_files(module_path: Path, excludes: Iterable[str]) -> List[Path]:
    test_files: List[Path] = []

    test_dirs = ["tests", "test", "__tests__"]
    for test_dir in test_dirs:
        path = module_path / test_dir
        if path.exists() and path.is_dir():
            for file in path.rglob("*"):
                if is_excluded(file, excludes):
                    continue
                if file.is_file():
                    test_files.append(file)

    test_files.extend(glob_files(module_path, PYTHON_TEST_GLOBS, excludes))
    test_files.extend(glob_files(module_path, NODE_TEST_GLOBS, excludes))
    test_files.extend(glob_files(module_path, GO_TEST_GLOBS, excludes))

    return sorted(set(test_files))


def build_record(module_path: Path, excludes: Iterable[str]) -> ModuleRecord:
    test_files = find_test_files(module_path, excludes)
    rust_inline = rust_inline_test_count(module_path, excludes)
    test_dirs = ["tests", "test", "__tests__"]
    has_tests_dir = any((module_path / name).exists() for name in test_dirs)

    return ModuleRecord(
        name=module_path.name,
        path=str(module_path),
        languages=detect_languages(module_path),
        has_readme=(module_path / "README.md").exists(),
        has_tests_dir=has_tests_dir,
        test_files=[str(p.relative_to(module_path)) for p in test_files],
        rust_inline_tests=rust_inline,
    )


def to_markdown(rows: List[ModuleRecord], include_files: bool) -> str:
    header = ["Module", "Lang", "README", "Tests Dir", "Test Files", "Rust Inline"]
    if include_files:
        header.append("Test File List")

    lines = ["| " + " | ".join(header) + " |", "|" + "|".join([" --- "] * len(header)) + "|"]

    for row in rows:
        langs = ", ".join(row.languages) if row.languages else "-"
        test_files_count = str(len(row.test_files))
        rust_inline = str(row.rust_inline_tests)
        cells = [
            row.name,
            langs,
            "yes" if row.has_readme else "no",
            "yes" if row.has_tests_dir else "no",
            test_files_count,
            rust_inline,
        ]
        if include_files:
            cells.append(", ".join(row.test_files) if row.test_files else "-")
        lines.append("| " + " | ".join(cells) + " |")

    return "\n".join(lines) + "\n"


def write_csv(rows: List[ModuleRecord], out, delimiter: str) -> None:
    fieldnames = [
        "module",
        "path",
        "languages",
        "has_readme",
        "has_tests_dir",
        "test_files",
        "rust_inline_tests",
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
                "has_tests_dir": row.has_tests_dir,
                "test_files": ",".join(row.test_files),
                "rust_inline_tests": row.rust_inline_tests,
            }
        )


def main() -> int:
    args = parse_args()
    root = Path(args.root).resolve() if args.root else repo_root_from_script()

    excludes = set(args.excludes or [])
    if not args.no_default_excludes:
        excludes |= DEFAULT_EXCLUDES

    modules = [
        path for path in root.iterdir() if path.name not in excludes and is_module_dir(path)
    ]
    records = [build_record(path, excludes) for path in sorted(modules, key=lambda p: p.name)]

    output = ""
    if args.format == "markdown":
        output = to_markdown(records, include_files=args.include_files)
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
