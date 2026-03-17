from __future__ import annotations

from datetime import datetime, timezone

from tests.helpers import build_store_from_journal
from tests.mutating_http_helpers import p, run_cli_mutating_http


NOW = 1_700_000_444.0
TASK_A = "A7h5eCi24RvAWKC3Hv3muf"
TASK_B = "KGvAPpMrzHAKMdgMiERP1V"
TASK_C = "MpkEei6ybkFS2n6SXvwfLf"


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


def test_before_after_inbox_payloads() -> None:
    store = build_store_from_journal(
        [
            _task(TASK_A, "A", ix=1024),
            _task(TASK_B, "B", ix=2048),
            _task(TASK_C, "C", ix=3072),
        ]
    )

    before = run_cli_mutating_http(
        f"reorder {TASK_C} --before {TASK_B}",
        store,
        extra_patches=[p("things_cloud.client.time.time", return_value=NOW)],
    )
    assert before.commits[0].payload == {
        TASK_C: {"t": 1, "e": "Task6", "p": {"ix": 1536, "md": NOW}}
    }

    after = run_cli_mutating_http(
        f"reorder {TASK_A} --after {TASK_B}",
        store,
        extra_patches=[p("things_cloud.client.time.time", return_value=NOW)],
    )
    assert after.commits[0].payload == {
        TASK_A: {"t": 1, "e": "Task6", "p": {"ix": 2560, "md": NOW}}
    }


def test_rebalance_payloads_and_ancestors() -> None:
    store = build_store_from_journal(
        [
            _task(TASK_A, "A", ix=1024),
            _task(TASK_B, "B", ix=1025),
            _task(TASK_C, "C", ix=1026),
        ]
    )
    result = run_cli_mutating_http(
        f"reorder {TASK_C} --after {TASK_A}",
        store,
        initial_head_index=50,
        extra_patches=[p("things_cloud.client.time.time", return_value=NOW)],
    )

    assert [c.ancestor_index for c in result.commits] == [50, 51]
    assert result.commits[0].payload == {
        TASK_C: {"t": 1, "e": "Task6", "p": {"ix": 2048, "md": NOW}}
    }
    assert result.commits[1].payload == {
        TASK_B: {"t": 1, "e": "Task6", "p": {"ix": 3072, "md": NOW}}
    }


def test_today_payload() -> None:
    today = _today_ts()
    store = build_store_from_journal(
        [
            _task(TASK_A, "A", st=1, sr=today, tir=today, ti=10, ix=100),
            _task(TASK_B, "B", st=1, sr=today, tir=today, ti=20, ix=200),
        ]
    )
    result = run_cli_mutating_http(
        f"reorder {TASK_A} --after {TASK_B}",
        store,
        extra_patches=[p("things_cloud.client.time.time", return_value=NOW)],
    )
    assert result.commits[0].payload == {
        TASK_A: {"t": 1, "e": "Task6", "p": {"tir": today, "ti": 21, "md": NOW}}
    }
