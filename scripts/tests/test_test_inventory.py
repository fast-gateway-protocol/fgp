import sys
from pathlib import Path

sys.path.append(str(Path(__file__).resolve().parents[1]))

import test_inventory as ti  # noqa: E402


def test_is_module_dir_with_readme(tmp_path: Path) -> None:
    module = tmp_path / "module"
    module.mkdir()
    (module / "README.md").write_text("readme", encoding="utf-8")
    assert ti.is_module_dir(module) is True


def test_detect_languages_rust(tmp_path: Path) -> None:
    module = tmp_path / "module"
    module.mkdir()
    (module / "Cargo.toml").write_text("[package]\nname = \"demo\"\n", encoding="utf-8")
    assert ti.detect_languages(module) == ["rust"]


def test_rust_inline_test_count(tmp_path: Path) -> None:
    module = tmp_path / "module"
    src = module / "src"
    src.mkdir(parents=True)
    (src / "lib.rs").write_text("#[test]\nfn it_works() {}\n", encoding="utf-8")
    assert ti.rust_inline_test_count(module, []) == 1


def test_find_test_files_in_tests_dir(tmp_path: Path) -> None:
    module = tmp_path / "module"
    tests_dir = module / "tests"
    tests_dir.mkdir(parents=True)
    test_file = tests_dir / "test_sample.py"
    test_file.write_text("def test_ok():\n    assert True\n", encoding="utf-8")

    files = ti.find_test_files(module, [])
    assert test_file in files
