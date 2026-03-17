from __future__ import annotations

import io
import re
import shlex
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch

import cli
from things_cloud.log_cache import _fold_item
from things_cloud.store import ThingsStore


_ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")


class _FakeClient:
    def __init__(self, email: str, password: str) -> None:
        self.email = email
        self.password = password


def build_store_from_journal(journal: list[dict]) -> ThingsStore:
    state: dict[str, dict] = {}
    for entry in journal:
        _fold_item(entry, state)
    return ThingsStore(state)


def run_cli(args: str, store: ThingsStore) -> str:
    argv = ["cli.py", "--no-color", "--no-sync", *shlex.split(args)]
    stdout = io.StringIO()

    with patch("cli.load_auth", return_value=("test@example.com", "secret")):
        with patch("cli.ThingsCloudClient", _FakeClient):
            with patch("cli.fold_state_from_append_log", return_value={}):
                with patch("cli.get_state_with_append_log", return_value={}):
                    with patch("cli.ThingsStore", return_value=store):
                        with patch("sys.argv", argv):
                            with redirect_stdout(stdout):
                                cli.main()

    return _ANSI_RE.sub("", stdout.getvalue())


def get_fixture(name: str) -> str:
    fixture_path = Path(__file__).parent / "fixtures" / f"{name}.txt"
    return fixture_path.read_text(encoding="utf-8")
