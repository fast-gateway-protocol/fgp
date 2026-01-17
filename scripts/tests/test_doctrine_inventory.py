import sys
from pathlib import Path

sys.path.append(str(Path(__file__).resolve().parents[1]))

import doctrine_inventory as di  # noqa: E402


def test_extract_headings() -> None:
    text = "# Title\n\n## Purpose\nBody\n### Scope\nMore\nNot a heading"
    headings = di.extract_headings(text)
    assert headings == ["Title", "Purpose", "Scope"]


def test_find_doctrine_files_dedupes(tmp_path: Path) -> None:
    module = tmp_path / "module"
    module.mkdir()
    (module / "README.md").write_text("readme", encoding="utf-8")
    (module / "DOCTRINE.md").write_text("# Purpose\n", encoding="utf-8")
    docs = module / "docs"
    docs.mkdir()
    (docs / "doctrine.md").write_text("# Scope\n", encoding="utf-8")

    found = di.find_doctrine_files(module)
    assert len(found) == 2
    assert any(path.name.lower() == "doctrine.md" for path in found)
    assert any(path.parent.name == "docs" for path in found)


def test_build_record_missing_headings(tmp_path: Path) -> None:
    module = tmp_path / "module"
    module.mkdir()
    (module / "README.md").write_text("readme", encoding="utf-8")
    (module / "Cargo.toml").write_text("[package]\nname = \"demo\"\n", encoding="utf-8")
    (module / "DOCTRINE.md").write_text("# Purpose\n# Tenets\n", encoding="utf-8")

    record = di.build_record(module, ["Purpose", "Scope", "Tenets"])
    assert record.name == "module"
    assert record.has_readme is True
    assert record.languages == ["rust"]
    assert record.missing_headings == ["Scope"]


def test_to_markdown_includes_missing_headings() -> None:
    record = di.ModuleRecord(
        name="demo",
        path="/tmp/demo",
        languages=["rust"],
        has_readme=True,
        has_docs_dir=False,
        has_src_dir=True,
        doctrine_files=["DOCTRINE.md"],
        headings=["Purpose"],
        missing_headings=["Scope"],
    )
    output = di.to_markdown([record], include_headings=True)
    assert "Missing Headings" in output
    assert "Scope" in output
