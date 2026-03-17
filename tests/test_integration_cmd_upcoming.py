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
        _task_create(
            "RTKdozwkfpXDhugqJBb9A5", "Prepare quarterly plan", ix=10, st=1, sr=day_a
        ),
        _task_create(
            "ShuDMT6hGzqxdTjt9ajstS", "Send kickoff email", ix=20, st=1, sr=day_a
        ),
        _task_create(
            "RbCMxciUzSDVCYYXVjAgVD", "Book team retro", ix=30, st=1, sr=day_b
        ),
        _task_create(
            "UAM9jermS7i1WZG7ky7kMg",
            "Should not show (scheduled today)",
            ix=40,
            st=1,
            sr=today,
        ),
        _task_create(
            "AnVsD8K9ZUNbjkeVsEmeZy",
            "Should not show (scheduled in past)",
            ix=50,
            st=1,
            sr=_day_ts(2000, 1, 1),
        ),
        _task_create(
            "Vdi8v72EndV5jM54gSHbRy",
            "Should not show (someday unscheduled)",
            ix=60,
            st=2,
        ),
        _task_create(
            "XNUinUJXsc5dZFXNuN6ya",
            "Should not show (backlog unscheduled)",
            ix=70,
            st=1,
        ),
    ]

    store = store_from_journal(journal)
    assert run_cli("upcoming", store) == get_fixture("upcoming_basic_grouped")


def test_upcoming_detailed_with_notes_and_checklist(store_from_journal) -> None:
    day_a = _day_ts(2099, 2, 10)
    journal = [
        _task_create(
            "6dVPrZf6uNWQHuMC1hBiqD",
            "Finalize launch plan",
            ix=10,
            st=1,
            sr=day_a,
            nt={"_t": "tx", "t": 1, "v": "Draft timeline\nConfirm stakeholders"},
        ),
        _checklist_create(
            "35juo14353ZfzTQYGktM4V",
            "6dVPrZf6uNWQHuMC1hBiqD",
            "Write announcement",
            ix=1,
            ss=0,
        ),
        _checklist_create(
            "H7s8p6ctuRWL2qLxZEqSod",
            "6dVPrZf6uNWQHuMC1hBiqD",
            "Get legal sign-off",
            ix=2,
            ss=3,
        ),
        _task_create(
            "2Lyo713TeXmyG7PXve3KS6",
            "Should not show (someday unscheduled)",
            ix=20,
            st=2,
        ),
        _task_create(
            "K6qWQRHTKymf8cdRFxMBwz",
            "Should not show (past scheduled)",
            ix=30,
            st=1,
            sr=_day_ts(2001, 1, 1),
        ),
    ]

    store = store_from_journal(journal)
    assert run_cli("upcoming --detailed", store) == get_fixture("upcoming_detailed")
