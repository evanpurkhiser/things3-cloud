from datetime import datetime, timedelta, timezone

from tests.helpers import get_fixture, run_cli


def _day_ts(offset_days: int = 0) -> int:
    return int(
        (datetime.now(tz=timezone.utc) + timedelta(days=offset_days))
        .replace(hour=0, minute=0, second=0, microsecond=0)
        .timestamp()
    )


def _task_create(
    uuid: str,
    title: str,
    *,
    ix: int,
    st: int = 1,
    ss: int = 0,
    sr: int | None = None,
    tr: bool = False,
    nt: str | dict | None = None,
) -> dict:
    props = {
        "tt": title,
        "tp": 0,
        "ss": ss,
        "st": st,
        "ix": ix,
        "cd": 1,
        "md": 1,
    }
    if sr is not None:
        props["sr"] = sr
    if tr:
        props["tr"] = True
    if nt is not None:
        props["nt"] = nt
    return {uuid: {"t": 0, "e": "Task6", "p": props}}


def _checklist_create(
    uuid: str,
    task_uuid: str,
    title: str,
    *,
    ix: int,
    ss: int = 0,
) -> dict:
    return {
        uuid: {
            "t": 0,
            "e": "ChecklistItem3",
            "p": {"tt": title, "ts": [task_uuid], "ss": ss, "ix": ix, "cd": 1, "md": 1},
        }
    }


def test_anytime_empty(store_from_journal) -> None:
    assert run_cli("anytime", store_from_journal([])) == get_fixture("anytime_empty")


def test_anytime_basic_list(store_from_journal) -> None:
    day_ts = _day_ts()
    journal = [
        _task_create("a-task-0000", "Draft roadmap", ix=10),
        _task_create("b-task-0000", "Pay rent", ix=20, sr=day_ts),
    ]

    assert run_cli("anytime", store_from_journal(journal)) == get_fixture(
        "anytime_basic"
    )


def test_anytime_filters_someday_future_trashed_and_completed(
    store_from_journal,
) -> None:
    journal = [
        _task_create("a-task-0000", "Visible anytime task", ix=10),
        _task_create("b-task-0000", "Someday backlog", ix=20, st=2),
        _task_create("c-task-0000", "Future scheduled", ix=30, sr=_day_ts(1)),
        _task_create("d-task-0000", "Trashed task", ix=40, tr=True),
        _task_create("e-task-0000", "Completed task", ix=50, ss=3),
    ]

    assert run_cli("anytime", store_from_journal(journal)) == get_fixture(
        "anytime_filtered"
    )


def test_anytime_detailed_with_notes_and_checklist(store_from_journal) -> None:
    journal = [
        _task_create(
            "a-task-0000",
            "Prepare trip plan",
            ix=10,
            nt={"_t": "tx", "t": 1, "v": "Book train tickets\nPack carry-on only"},
        ),
        _checklist_create(
            "x-item-0001", "a-task-0000", "Confirm passport expiry", ix=1, ss=0
        ),
        _checklist_create(
            "y-item-0001", "a-task-0000", "Download offline maps", ix=2, ss=3
        ),
    ]

    assert run_cli("anytime --detailed", store_from_journal(journal)) == get_fixture(
        "anytime_detailed"
    )
