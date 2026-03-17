from datetime import datetime, timezone

from tests.helpers import get_fixture, run_cli


def _day_ts(year: int, month: int, day: int) -> int:
    return int(datetime(year, month, day, tzinfo=timezone.utc).timestamp())


def _today_ts() -> int:
    return int(
        datetime.now(tz=timezone.utc)
        .replace(hour=0, minute=0, second=0, microsecond=0)
        .timestamp()
    )


def _task_create(
    uuid: str,
    title: str,
    *,
    ix: int,
    st: int,
    sr: int | None = None,
    nt: str | dict | None = None,
) -> dict:
    props = {
        "tt": title,
        "tp": 0,
        "ss": 0,
        "st": st,
        "ix": ix,
        "cd": 1,
        "md": 1,
    }
    if sr is not None:
        props["sr"] = sr
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


def test_upcoming_empty(store_from_journal) -> None:
    store = store_from_journal([])
    assert run_cli("upcoming", store) == get_fixture("upcoming_empty")


def test_upcoming_basic_grouped_by_future_date(store_from_journal) -> None:
    day_a = _day_ts(2099, 1, 5)
    day_b = _day_ts(2099, 1, 7)
    today = _today_ts()
    journal = [
        _task_create("a-task-0000", "Prepare quarterly plan", ix=10, st=1, sr=day_a),
        _task_create("b-task-0000", "Send kickoff email", ix=20, st=1, sr=day_a),
        _task_create("c-task-0000", "Book team retro", ix=30, st=1, sr=day_b),
        _task_create(
            "d-task-0000", "Should not show (scheduled today)", ix=40, st=1, sr=today
        ),
        _task_create(
            "e-task-0000",
            "Should not show (scheduled in past)",
            ix=50,
            st=1,
            sr=_day_ts(2000, 1, 1),
        ),
        _task_create(
            "f-task-0000", "Should not show (someday unscheduled)", ix=60, st=2
        ),
        _task_create(
            "g-task-0000", "Should not show (backlog unscheduled)", ix=70, st=1
        ),
    ]

    store = store_from_journal(journal)
    assert run_cli("upcoming", store) == get_fixture("upcoming_basic_grouped")


def test_upcoming_detailed_with_notes_and_checklist(store_from_journal) -> None:
    day_a = _day_ts(2099, 2, 10)
    journal = [
        _task_create(
            "h-task-0000",
            "Finalize launch plan",
            ix=10,
            st=1,
            sr=day_a,
            nt={"_t": "tx", "t": 1, "v": "Draft timeline\nConfirm stakeholders"},
        ),
        _checklist_create(
            "x-item-0001", "h-task-0000", "Write announcement", ix=1, ss=0
        ),
        _checklist_create(
            "y-item-0001", "h-task-0000", "Get legal sign-off", ix=2, ss=3
        ),
        _task_create(
            "i-task-0000", "Should not show (someday unscheduled)", ix=20, st=2
        ),
        _task_create(
            "j-task-0000",
            "Should not show (past scheduled)",
            ix=30,
            st=1,
            sr=_day_ts(2001, 1, 1),
        ),
    ]

    store = store_from_journal(journal)
    assert run_cli("upcoming --detailed", store) == get_fixture("upcoming_detailed")
