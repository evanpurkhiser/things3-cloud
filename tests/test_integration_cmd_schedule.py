from __future__ import annotations

from datetime import datetime, timezone

from things_cloud.cli.common import _day_to_timestamp
from tests.mutating_fixtures import store, task, today_ts
from tests.mutating_http_helpers import assert_commit_payloads, p, run_cli_mutating_http


NOW = 1_700_000_333.0
TASK_UUID = "A7h5eCi24RvAWKC3Hv3muf"


def test_when_variants_payloads() -> None:
    test_store = store(task(TASK_UUID, "Schedule me"))
    today = today_ts()
    future_ts = _day_to_timestamp(datetime(2099, 5, 10, tzinfo=timezone.utc))
    cases = [
        ("today", {"st": 1, "sr": today, "tir": today, "sb": 0}),
        ("someday", {"st": 2, "sr": None, "tir": None, "sb": 0}),
        ("anytime", {"st": 1, "sr": None, "tir": None, "sb": 0}),
        ("evening", {"st": 1, "sr": today, "tir": today, "sb": 1}),
        ("2099-05-10", {"st": 2, "sr": future_ts, "tir": future_ts, "sb": 0}),
    ]

    for when, update in cases:
        result = run_cli_mutating_http(
            f"schedule {TASK_UUID} --when {when}",
            test_store,
            extra_patches=[p("things_cloud.client.time.time", return_value=NOW)],
        )
        expected = dict(update)
        expected["md"] = NOW
        assert_commit_payloads(
            result,
            {TASK_UUID: {"t": 1, "e": "Task6", "p": expected}},
        )


def test_deadline_and_clear_deadline_payloads() -> None:
    test_store = store(task(TASK_UUID, "Schedule me"))
    deadline_ts = _day_to_timestamp(datetime(2034, 2, 1, tzinfo=timezone.utc))

    deadline = run_cli_mutating_http(
        f"schedule {TASK_UUID} --deadline 2034-02-01",
        test_store,
        extra_patches=[p("things_cloud.client.time.time", return_value=NOW)],
    )
    assert_commit_payloads(
        deadline,
        {TASK_UUID: {"t": 1, "e": "Task6", "p": {"dd": deadline_ts, "md": NOW}}},
    )

    clear = run_cli_mutating_http(
        f"schedule {TASK_UUID} --clear-deadline",
        test_store,
        extra_patches=[p("things_cloud.client.time.time", return_value=NOW)],
    )
    assert_commit_payloads(
        clear,
        {TASK_UUID: {"t": 1, "e": "Task6", "p": {"dd": None, "md": NOW}}},
    )
