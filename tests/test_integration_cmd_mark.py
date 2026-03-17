from __future__ import annotations

from tests.mutating_fixtures import checklist, store, task
from tests.mutating_http_helpers import assert_commit_payloads, p, run_cli_mutating_http


NOW = 1_700_000_111.0
TASK_A = "A7h5eCi24RvAWKC3Hv3muf"
TASK_B = "KGvAPpMrzHAKMdgMiERP1V"
CHECK_A = "MpkEei6ybkFS2n6SXvwfLf"
CHECK_B = "JFdhhhp37fpryAKu8UXwzK"


def test_done_single_payload() -> None:
    test_store = store(task(TASK_A, "Alpha"))
    result = run_cli_mutating_http(
        f"mark {TASK_A} --done",
        test_store,
        extra_patches=[
            p("things_cloud.cli.cmd_mark.time.time", return_value=NOW),
            p("things_cloud.client.time.time", return_value=NOW),
        ],
    )
    assert_commit_payloads(
        result,
        {TASK_A: {"t": 1, "e": "Task6", "p": {"ss": 3, "sp": NOW, "md": NOW}}},
    )


def test_done_multi_payload() -> None:
    test_store = store(task(TASK_A, "Alpha"), task(TASK_B, "Beta"))
    result = run_cli_mutating_http(
        f"mark {TASK_A} {TASK_B} --done",
        test_store,
        extra_patches=[
            p("things_cloud.cli.cmd_mark.time.time", return_value=NOW),
            p("things_cloud.client.time.time", return_value=NOW),
        ],
    )
    assert_commit_payloads(
        result,
        {
            TASK_A: {"t": 1, "e": "Task6", "p": {"ss": 3, "sp": NOW, "md": NOW}},
            TASK_B: {"t": 1, "e": "Task6", "p": {"ss": 3, "sp": NOW, "md": NOW}},
        },
    )


def test_incomplete_payload() -> None:
    test_store = store(task(TASK_A, "Alpha", ss=3))
    result = run_cli_mutating_http(
        f"mark {TASK_A} --incomplete",
        test_store,
        extra_patches=[
            p("things_cloud.cli.cmd_mark.time.time", return_value=NOW),
            p("things_cloud.client.time.time", return_value=NOW),
        ],
    )
    assert_commit_payloads(
        result,
        {TASK_A: {"t": 1, "e": "Task6", "p": {"ss": 0, "sp": None, "md": NOW}}},
    )


def test_canceled_payload() -> None:
    test_store = store(task(TASK_A, "Alpha"))
    result = run_cli_mutating_http(
        f"mark {TASK_A} --canceled",
        test_store,
        extra_patches=[
            p("things_cloud.cli.cmd_mark.time.time", return_value=NOW),
            p("things_cloud.client.time.time", return_value=NOW),
        ],
    )
    assert_commit_payloads(
        result,
        {TASK_A: {"t": 1, "e": "Task6", "p": {"ss": 2, "sp": NOW, "md": NOW}}},
    )


def test_checklist_check_uncheck_cancel_payloads() -> None:
    test_store = store(
        task(TASK_A, "Task with checklist"),
        checklist(CHECK_A, TASK_A, "One", ix=1),
        checklist(CHECK_B, TASK_A, "Two", ix=2),
    )

    checked = run_cli_mutating_http(
        f"mark {TASK_A} --check {CHECK_A[:6]},{CHECK_B[:6]}",
        test_store,
        extra_patches=[p("things_cloud.cli.cmd_mark.time.time", return_value=NOW)],
    )
    assert_commit_payloads(
        checked,
        {
            CHECK_A: {
                "t": 1,
                "e": "ChecklistItem3",
                "p": {"ss": 3, "sp": NOW, "md": NOW},
            },
            CHECK_B: {
                "t": 1,
                "e": "ChecklistItem3",
                "p": {"ss": 3, "sp": NOW, "md": NOW},
            },
        },
    )

    unchecked = run_cli_mutating_http(
        f"mark {TASK_A} --uncheck {CHECK_A[:6]}",
        test_store,
        extra_patches=[p("things_cloud.cli.cmd_mark.time.time", return_value=NOW)],
    )
    assert_commit_payloads(
        unchecked,
        {
            CHECK_A: {
                "t": 1,
                "e": "ChecklistItem3",
                "p": {"ss": 0, "sp": None, "md": NOW},
            }
        },
    )

    canceled = run_cli_mutating_http(
        f"mark {TASK_A} --check-cancel {CHECK_B[:6]}",
        test_store,
        extra_patches=[p("things_cloud.cli.cmd_mark.time.time", return_value=NOW)],
    )
    assert_commit_payloads(
        canceled,
        {
            CHECK_B: {
                "t": 1,
                "e": "ChecklistItem3",
                "p": {"ss": 2, "sp": NOW, "md": NOW},
            }
        },
    )
