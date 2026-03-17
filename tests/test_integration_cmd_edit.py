from __future__ import annotations

from things_cloud.cli.common import _task6_note
from tests.helpers import build_store_from_journal
from tests.mutating_http_helpers import p, run_cli_mutating_http


NOW = 1_700_000_222.0
TASK_UUID = "A7h5eCi24RvAWKC3Hv3muf"
PROJECT_UUID = "KGvAPpMrzHAKMdgMiERP1V"
AREA_UUID = "MpkEei6ybkFS2n6SXvwfLf"


def _task(uuid: str, title: str, **props) -> dict:
    base = {"tt": title, "tp": 0, "ss": 0, "st": 0, "ix": 0, "cd": 1, "md": 1}
    base.update(props)
    return {uuid: {"t": 0, "e": "Task6", "p": base}}


def _project(uuid: str, title: str, **props) -> dict:
    base = {"tt": title, "tp": 1, "ss": 0, "st": 1, "ix": 0, "cd": 1, "md": 1}
    base.update(props)
    return {uuid: {"t": 0, "e": "Task6", "p": base}}


def _area(uuid: str, title: str, **props) -> dict:
    base = {"tt": title, "ix": 0}
    base.update(props)
    return {uuid: {"t": 0, "e": "Area3", "p": base}}


def test_title_notes_and_clear_notes_payload() -> None:
    store = build_store_from_journal([_task(TASK_UUID, "Old title")])

    title = run_cli_mutating_http(
        f'edit {TASK_UUID} --title "New title"',
        store,
        extra_patches=[p("things_cloud.client.time.time", return_value=NOW)],
    )
    assert title.commits[0].payload == {
        TASK_UUID: {"t": 1, "e": "Task6", "p": {"tt": "New title", "md": NOW}}
    }

    notes = run_cli_mutating_http(
        f'edit {TASK_UUID} --notes "new notes"',
        store,
        extra_patches=[p("things_cloud.client.time.time", return_value=NOW)],
    )
    assert notes.commits[0].payload == {
        TASK_UUID: {
            "t": 1,
            "e": "Task6",
            "p": {"nt": _task6_note("new notes"), "md": NOW},
        }
    }

    clear = run_cli_mutating_http(
        f"edit {TASK_UUID} --notes ''",
        store,
        extra_patches=[p("things_cloud.client.time.time", return_value=NOW)],
    )
    assert clear.commits[0].payload == {
        TASK_UUID: {
            "t": 1,
            "e": "Task6",
            "p": {"nt": {"_t": "tx", "t": 1, "ch": 0, "v": ""}, "md": NOW},
        }
    }


def test_move_targets_payload() -> None:
    journal = [
        _task(TASK_UUID, "Movable", st=0),
        _project(PROJECT_UUID, "Roadmap"),
        _area(AREA_UUID, "Work"),
    ]
    store = build_store_from_journal(journal)

    inbox = run_cli_mutating_http(
        f"edit {TASK_UUID} --move inbox",
        store,
        extra_patches=[p("things_cloud.client.time.time", return_value=NOW)],
    )
    assert inbox.commits[0].payload == {
        TASK_UUID: {
            "t": 1,
            "e": "Task6",
            "p": {
                "pr": [],
                "ar": [],
                "agr": [],
                "st": 0,
                "sr": None,
                "tir": None,
                "sb": 0,
                "md": NOW,
            },
        }
    }

    clear = run_cli_mutating_http(
        f"edit {TASK_UUID} --move clear",
        store,
        extra_patches=[p("things_cloud.client.time.time", return_value=NOW)],
    )
    assert clear.commits[0].payload == {
        TASK_UUID: {
            "t": 1,
            "e": "Task6",
            "p": {"pr": [], "ar": [], "agr": [], "st": 1, "md": NOW},
        }
    }

    project_move = run_cli_mutating_http(
        f"edit {TASK_UUID} --move {PROJECT_UUID}",
        store,
        extra_patches=[p("things_cloud.client.time.time", return_value=NOW)],
    )
    assert project_move.commits[0].payload == {
        TASK_UUID: {
            "t": 1,
            "e": "Task6",
            "p": {"pr": [PROJECT_UUID], "ar": [], "agr": [], "st": 1, "md": NOW},
        }
    }

    area_move = run_cli_mutating_http(
        f"edit {TASK_UUID} --move {AREA_UUID}",
        store,
        extra_patches=[p("things_cloud.client.time.time", return_value=NOW)],
    )
    assert area_move.commits[0].payload == {
        TASK_UUID: {
            "t": 1,
            "e": "Task6",
            "p": {"ar": [AREA_UUID], "pr": [], "agr": [], "st": 1, "md": NOW},
        }
    }
