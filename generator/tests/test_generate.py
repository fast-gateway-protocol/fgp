import re
import sys
from pathlib import Path

sys.path.append(str(Path(__file__).resolve().parents[1]))

import generate  # noqa: E402


def test_to_pascal_case() -> None:
    assert generate.to_pascal_case("slack") == "Slack"
    assert generate.to_pascal_case("foo_bar") == "FooBar"
    assert generate.to_pascal_case("foo-bar") == "FooBar"
    assert generate.to_pascal_case("foo_bar-baz") == "FooBarBaz"


def test_to_snake_case() -> None:
    assert generate.to_snake_case("Slack") == "slack"
    assert generate.to_snake_case("Foo-Bar") == "foo_bar"
    assert generate.to_snake_case("foo_bar") == "foo_bar"


def test_get_date_format() -> None:
    date = generate.get_date()
    assert re.match(r"^\d{2}/\d{2}/\d{4}$", date)


def test_render_template_replaces_tokens() -> None:
    template = "Hello {{name}} from {{place}}!"
    context = {"name": "Ada", "place": "FGP"}
    assert generate.render_template(template, context) == "Hello Ada from FGP!"


def test_print_presets_output(capsys) -> None:
    presets = {
        "slack": {"category": "Communication", "description": "Chat", "env_token": "SLACK_TOKEN"},
        "notion": {"category": "Knowledge", "description": "Docs", "env_token": "NOTION_TOKEN"},
    }
    generate.print_presets(presets)
    output = capsys.readouterr().out
    assert "FGP Daemon Generator" in output
    assert "Communication" in output
    assert "Knowledge" in output
    assert "slack" in output
    assert "notion" in output
