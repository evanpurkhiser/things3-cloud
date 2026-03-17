from __future__ import annotations

from tests.helpers import build_store_from_journal
from tests.mutating_http_helpers import run_cli_mutating_http


TASK_A = "A7h5eCi24RvAWKC3Hv3muf"
TASK_B = "KGvAPpMrzHAKMdgMiERP1V"
AREA_A = "MpkEei6ybkFS2n6SXvwfLf"


def _task(uuid: str, title: str, **props) -> dict:
    base = {"tt": title, "tp": 0, "ss": 0, "st": 0, "ix": 0, "cd": 1, "md": 1}
    base.update(props)
    return {uuid: {"t": 0, "e": "Task6", "p": base}}


def _area(uuid: str, title: str, **props) -> dict:
    base = {"tt": title, "ix": 0}
    base.update(props)
    return {uuid: {"t": 0, "e": "Area3", "p": base}}


def test_single_payload() -> None:
    store = build_store_from_journal([_task(TASK_A, "Alpha")])
    result = run_cli_mutating_http(f"delete {TASK_A}", store)
    assert result.commits[0].payload == {TASK_A: {"t": 2, "e": "Task6", "p": {}}}


def test_multiple_payload() -> None:
    store = build_store_from_journal([_task(TASK_A, "Alpha"), _area(AREA_A, "Work")])
    result = run_cli_mutating_http(f"delete {TASK_A} {AREA_A}", store)
    assert result.commits[0].payload == {
        TASK_A: {"t": 2, "e": "Task6", "p": {}},
        AREA_A: {"t": 2, "e": "Area3", "p": {}},
    }


def test_skip_already_trashed_payload() -> None:
    store = build_store_from_journal(
        [
            _task(TASK_A, "Active"),
            _task(TASK_B, "Trashed", tr=True),
        ]
    )
    result = run_cli_mutating_http(f"delete {TASK_A} {TASK_B}", store)
    assert len(result.commits) == 1
    assert result.commits[0].payload == {TASK_A: {"t": 2, "e": "Task6", "p": {}}}
