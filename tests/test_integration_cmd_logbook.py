from datetime import datetime, timezone

import pytest

from tests.helpers import get_fixture, run_cli


def _ts(year: int, month: int, day: int, hour: int, minute: int = 0) -> int:
    return int(
        datetime(year, month, day, hour, minute, tzinfo=timezone.utc).timestamp()
    )


def _task(
    uuid: str,
    title: str,
    *,
    status: int,
    ix: int,
    stop_ts: int,
    notes: str | None = None,
) -> dict:
    props = {
        "tt": title,
        "st": 1,
        "ss": status,
        "ix": ix,
        "sp": stop_ts,
        "cd": 1,
        "md": 1,
    }
    if notes is not None:
        props["nt"] = {"_t": "tx", "t": 1, "ch": 0, "v": notes}
    return {uuid: {"t": 0, "e": "Task6", "p": props}}


def _checklist_item(
    uuid: str,
    task_uuid: str,
    title: str,
    *,
    ix: int,
    status: int = 0,
) -> dict:
    return {
        uuid: {
            "t": 0,
            "e": "ChecklistItem3",
            "p": {
                "tt": title,
                "ts": [task_uuid],
                "ss": status,
                "ix": ix,
                "cd": 1,
                "md": 1,
            },
        }
    }


def _base_journal() -> list[dict]:
    return [
        _task(
            "done-a-0001",
            "Ship release",
            status=3,
            ix=10,
            stop_ts=_ts(2025, 3, 15, 15, 45),
        ),
        _task(
            "cancel-b-0002",
            "Skip sync",
            status=2,
            ix=20,
            stop_ts=_ts(2025, 3, 15, 9, 0),
        ),
        _task(
            "done-c-0003",
            "Archive notes",
            status=3,
            ix=30,
            stop_ts=_ts(2025, 3, 14, 18, 30),
        ),
    ]


def test_logbook_empty(store_from_journal) -> None:
    assert run_cli("logbook", store_from_journal([])) == get_fixture("logbook_empty")


def test_logbook_groups_by_completion_day_with_completed_and_canceled_items(
    store_from_journal,
) -> None:
    assert run_cli("logbook", store_from_journal(_base_journal())) == get_fixture(
        "logbook_basic_grouped"
    )


@pytest.mark.parametrize(
    ("args", "fixture_name"),
    [
        ("logbook --from 2025-03-15", "logbook_from_filter"),
        ("logbook --to 2025-03-14", "logbook_to_filter"),
    ],
)
def test_logbook_date_filter_options(
    store_from_journal, args: str, fixture_name: str
) -> None:
    assert run_cli(args, store_from_journal(_base_journal())) == get_fixture(
        fixture_name
    )


def test_logbook_detailed_mode_renders_notes_and_checklist(store_from_journal) -> None:
    journal = [
        _task(
            "detail-a-0001",
            "Write retro",
            status=3,
            ix=10,
            stop_ts=_ts(2025, 3, 16, 8, 0),
            notes="Capture wins\nCapture risks",
        ),
        _task(
            "detail-b-0002",
            "Pack launch checklist",
            status=3,
            ix=20,
            stop_ts=_ts(2025, 3, 16, 7, 30),
            notes="Before handoff",
        ),
        _checklist_item("item-a-0001", "detail-b-0002", "Verify docs", ix=10),
        _checklist_item("item-b-0002", "detail-b-0002", "Post update", ix=20, status=3),
    ]

    assert run_cli("logbook --detailed", store_from_journal(journal)) == get_fixture(
        "logbook_detailed"
    )
