from __future__ import annotations

from datetime import datetime, timezone

from things_cloud.cli.common import _day_to_timestamp
from tests.helpers import build_store_from_journal
from tests.mutating_http_helpers import p, run_cli_mutating_http


NOW = 1_700_000_333.0
TASK_UUID = "A7h5eCi24RvAWKC3Hv3muf"


def _today_ts() -> int:
    return int(
        datetime.now(tz=timezone.utc)
        .replace(hour=0, minute=0, second=0, microsecond=0)
        .timestamp()
    )


def _task(uuid: str, title: str, **props) -> dict:
    base = {"tt": title, "tp": 0, "ss": 0, "st": 0, "ix": 0, "cd": 1, "md": 1}
    base.update(props)
    return {uuid: {"t": 0, "e": "Task6", "p": base}}


def test_integration_cmd_schedule_when_variants_payloads() -> None:
    store = build_store_from_journal([_task(TASK_UUID, "Schedule me")])
    today = _today_ts()
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
            store,
            extra_patches=[p("things_cloud.client.time.time", return_value=NOW)],
        )
        expected = dict(update)
        expected["md"] = NOW
        assert result.commits[0].payload == {
            TASK_UUID: {"t": 1, "e": "Task6", "p": expected}
        }


def test_integration_cmd_schedule_deadline_and_clear_deadline_payloads() -> None:
    store = build_store_from_journal([_task(TASK_UUID, "Schedule me")])
    deadline_ts = _day_to_timestamp(datetime(2034, 2, 1, tzinfo=timezone.utc))

    deadline = run_cli_mutating_http(
        f"schedule {TASK_UUID} --deadline 2034-02-01",
        store,
        extra_patches=[p("things_cloud.client.time.time", return_value=NOW)],
    )
    assert deadline.commits[0].payload == {
        TASK_UUID: {"t": 1, "e": "Task6", "p": {"dd": deadline_ts, "md": NOW}}
    }

    clear = run_cli_mutating_http(
        f"schedule {TASK_UUID} --clear-deadline",
        store,
        extra_patches=[p("things_cloud.client.time.time", return_value=NOW)],
    )
    assert clear.commits[0].payload == {
        TASK_UUID: {"t": 1, "e": "Task6", "p": {"dd": None, "md": NOW}}
    }
