from __future__ import annotations

from tests.helpers import build_store_from_journal
from tests.mutating_http_helpers import p, run_cli_mutating_http


NOW = 1_700_000_111.0
TASK_A = "A7h5eCi24RvAWKC3Hv3muf"
TASK_B = "KGvAPpMrzHAKMdgMiERP1V"
CHECK_A = "MpkEei6ybkFS2n6SXvwfLf"
CHECK_B = "JFdhhhp37fpryAKu8UXwzK"


def _task(uuid: str, title: str, **props) -> dict:
    base = {"tt": title, "tp": 0, "ss": 0, "st": 0, "ix": 0, "cd": 1, "md": 1}
    base.update(props)
    return {uuid: {"t": 0, "e": "Task6", "p": base}}


def _checklist(uuid: str, task_uuid: str, title: str, **props) -> dict:
    base = {"tt": title, "ts": [task_uuid], "ss": 0, "ix": 0, "cd": 1, "md": 1}
    base.update(props)
    return {uuid: {"t": 0, "e": "ChecklistItem3", "p": base}}


def test_done_single_payload() -> None:
    store = build_store_from_journal([_task(TASK_A, "Alpha")])
    result = run_cli_mutating_http(
        f"mark {TASK_A} --done",
        store,
        extra_patches=[
            p("things_cloud.cli.cmd_mark.time.time", return_value=NOW),
            p("things_cloud.client.time.time", return_value=NOW),
        ],
    )
    assert result.commits[0].payload == {
        TASK_A: {"t": 1, "e": "Task6", "p": {"ss": 3, "sp": NOW, "md": NOW}}
    }


def test_done_multi_payload() -> None:
    store = build_store_from_journal([_task(TASK_A, "Alpha"), _task(TASK_B, "Beta")])
    result = run_cli_mutating_http(
        f"mark {TASK_A} {TASK_B} --done",
        store,
        extra_patches=[
            p("things_cloud.cli.cmd_mark.time.time", return_value=NOW),
            p("things_cloud.client.time.time", return_value=NOW),
        ],
    )
    assert result.commits[0].payload == {
        TASK_A: {"t": 1, "e": "Task6", "p": {"ss": 3, "sp": NOW, "md": NOW}},
        TASK_B: {"t": 1, "e": "Task6", "p": {"ss": 3, "sp": NOW, "md": NOW}},
    }


def test_incomplete_payload() -> None:
    store = build_store_from_journal([_task(TASK_A, "Alpha", ss=3)])
    result = run_cli_mutating_http(
        f"mark {TASK_A} --incomplete",
        store,
        extra_patches=[
            p("things_cloud.cli.cmd_mark.time.time", return_value=NOW),
            p("things_cloud.client.time.time", return_value=NOW),
        ],
    )
    assert result.commits[0].payload == {
        TASK_A: {"t": 1, "e": "Task6", "p": {"ss": 0, "sp": None, "md": NOW}}
    }


def test_canceled_payload() -> None:
    store = build_store_from_journal([_task(TASK_A, "Alpha")])
    result = run_cli_mutating_http(
        f"mark {TASK_A} --canceled",
        store,
        extra_patches=[
            p("things_cloud.cli.cmd_mark.time.time", return_value=NOW),
            p("things_cloud.client.time.time", return_value=NOW),
        ],
    )
    assert result.commits[0].payload == {
        TASK_A: {"t": 1, "e": "Task6", "p": {"ss": 2, "sp": NOW, "md": NOW}}
    }


def test_checklist_check_uncheck_cancel_payloads() -> None:
    journal = [
        _task(TASK_A, "Task with checklist"),
        _checklist(CHECK_A, TASK_A, "One", ix=1),
        _checklist(CHECK_B, TASK_A, "Two", ix=2),
    ]
    store = build_store_from_journal(journal)

    checked = run_cli_mutating_http(
        f"mark {TASK_A} --check {CHECK_A[:6]},{CHECK_B[:6]}",
        store,
        extra_patches=[p("things_cloud.cli.cmd_mark.time.time", return_value=NOW)],
    )
    assert checked.commits[0].payload == {
        CHECK_A: {"t": 1, "e": "ChecklistItem3", "p": {"ss": 3, "sp": NOW, "md": NOW}},
        CHECK_B: {"t": 1, "e": "ChecklistItem3", "p": {"ss": 3, "sp": NOW, "md": NOW}},
    }

    unchecked = run_cli_mutating_http(
        f"mark {TASK_A} --uncheck {CHECK_A[:6]}",
        store,
        extra_patches=[p("things_cloud.cli.cmd_mark.time.time", return_value=NOW)],
    )
    assert unchecked.commits[0].payload == {
        CHECK_A: {"t": 1, "e": "ChecklistItem3", "p": {"ss": 0, "sp": None, "md": NOW}}
    }

    canceled = run_cli_mutating_http(
        f"mark {TASK_A} --check-cancel {CHECK_B[:6]}",
        store,
        extra_patches=[p("things_cloud.cli.cmd_mark.time.time", return_value=NOW)],
    )
    assert canceled.commits[0].payload == {
        CHECK_B: {"t": 1, "e": "ChecklistItem3", "p": {"ss": 2, "sp": NOW, "md": NOW}}
    }
