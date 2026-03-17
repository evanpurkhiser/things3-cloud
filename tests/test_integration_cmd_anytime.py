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
        _task_create("6aXZoaKdhWbtkVDjkjSh6t", "Draft roadmap", ix=10),
        _task_create("3eyRB1WYUNtkYfE8B3MGPn", "Pay rent", ix=20, sr=day_ts),
    ]

    assert run_cli("anytime", store_from_journal(journal)) == get_fixture(
        "anytime_basic"
    )


def test_anytime_filters_someday_future_trashed_and_completed(
    store_from_journal,
) -> None:
    journal = [
        _task_create("6aXZoaKdhWbtkVDjkjSh6t", "Visible anytime task", ix=10),
        _task_create("JGHbpq9qT112kF3pMfHYVN", "Someday backlog", ix=20, st=2),
        _task_create(
            "EVm4iCcMXiBp4eWKojk2zp", "Future scheduled", ix=30, sr=_day_ts(1)
        ),
        _task_create("4LHrEe3jyYApPfnNPMPpxn", "Trashed task", ix=40, tr=True),
        _task_create("QSHXpCLatmt3h9DskZ1RMF", "Completed task", ix=50, ss=3),
    ]

    assert run_cli("anytime", store_from_journal(journal)) == get_fixture(
        "anytime_filtered"
    )


def test_anytime_detailed_with_notes_and_checklist(store_from_journal) -> None:
    journal = [
        _task_create(
            "6aXZoaKdhWbtkVDjkjSh6t",
            "Prepare trip plan",
            ix=10,
            nt={"_t": "tx", "t": 1, "v": "Book train tickets\nPack carry-on only"},
        ),
        _checklist_create(
            "LK55LNQ2Th3Tdx2qi161pM",
            "6aXZoaKdhWbtkVDjkjSh6t",
            "Confirm passport expiry",
            ix=1,
            ss=0,
        ),
        _checklist_create(
            "CwqFCJUboRLmL8E2D7J24f",
            "6aXZoaKdhWbtkVDjkjSh6t",
            "Download offline maps",
            ix=2,
            ss=3,
        ),
    ]

    assert run_cli("anytime --detailed", store_from_journal(journal)) == get_fixture(
        "anytime_detailed"
    )
