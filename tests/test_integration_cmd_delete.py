from __future__ import annotations

from tests.mutating_fixtures import area, store, task
from tests.mutating_http_helpers import (
    assert_commit_payloads,
    assert_no_commits,
    run_cli_mutating_http,
)


TASK_A = "A7h5eCi24RvAWKC3Hv3muf"
TASK_B = "KGvAPpMrzHAKMdgMiERP1V"
AREA_A = "MpkEei6ybkFS2n6SXvwfLf"


def test_single_payload() -> None:
    test_store = store(task(TASK_A, "Alpha"))
    result = run_cli_mutating_http(f"delete {TASK_A}", test_store)
    assert_commit_payloads(result, {TASK_A: {"t": 2, "e": "Task6", "p": {}}})


def test_multiple_payload() -> None:
    test_store = store(task(TASK_A, "Alpha"), area(AREA_A, "Work"))
    result = run_cli_mutating_http(f"delete {TASK_A} {AREA_A}", test_store)
    assert_commit_payloads(
        result,
        {
            TASK_A: {"t": 2, "e": "Task6", "p": {}},
            AREA_A: {"t": 2, "e": "Area3", "p": {}},
        },
    )


def test_skip_already_trashed_payload() -> None:
    test_store = store(
        task(TASK_A, "Active"),
        task(TASK_B, "Trashed", tr=True),
    )
    result = run_cli_mutating_http(f"delete {TASK_A} {TASK_B}", test_store)
    assert_commit_payloads(result, {TASK_A: {"t": 2, "e": "Task6", "p": {}}})


def test_missing_item_id_is_reported() -> None:
    result = run_cli_mutating_http("delete nope", store(task(TASK_A, "A")))
    assert_no_commits(result)
    assert result.stderr == "Item not found: nope\n"
