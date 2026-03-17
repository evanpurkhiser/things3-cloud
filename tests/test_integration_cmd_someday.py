from datetime import datetime, timezone

from tests.helpers import get_fixture, run_cli


def _future_day_ts() -> int:
    return (
        int(
            datetime.now(tz=timezone.utc)
            .replace(hour=0, minute=0, second=0, microsecond=0)
            .timestamp()
        )
        + 86400
    )


def _task_create(
    uuid: str,
    title: str,
    *,
    ix: int,
    st: int,
    tp: int = 0,
    sr: int | None = None,
    pr: list[str] | None = None,
    rr: dict | None = None,
    nt: str | dict | None = None,
) -> dict:
    props = {
        "tt": title,
        "tp": tp,
        "ss": 0,
        "st": st,
        "ix": ix,
        "cd": 1,
        "md": 1,
    }
    if sr is not None:
        props["sr"] = sr
    if pr is not None:
        props["pr"] = pr
    if rr is not None:
        props["rr"] = rr
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


def test_someday_empty(store_from_journal) -> None:
    store = store_from_journal([])
    assert run_cli("someday", store) == get_fixture("someday_empty")


def test_someday_mixed_projects_and_tasks_ordering(store_from_journal) -> None:
    journal = [
        _task_create("BpPQW1vgHjAhTQGmCnGno5", "Brainstorm blog ideas", ix=5, st=2),
        _task_create(
            "HFiZWtutsAB7u9GeDToisD", "Home office redesign", ix=10, st=2, tp=1
        ),
        _task_create("LgYa4JKjhGtSboGa59LaAS", "Try a new bread recipe", ix=15, st=2),
    ]

    store = store_from_journal(journal)
    assert run_cli("someday", store) == get_fixture("someday_mixed_ordering")


def test_someday_filters_future_scheduled_templates_and_project_tasks(
    store_from_journal,
) -> None:
    future_ts = _future_day_ts()
    journal = [
        _task_create("BpPQW1vgHjAhTQGmCnGno5", "Read design books", ix=5, st=2),
        _task_create("HFiZWtutsAB7u9GeDToisD", "Cabin renovation", ix=10, st=2, tp=1),
        _task_create(
            "LgYa4JKjhGtSboGa59LaAS", "Plan winter trip", ix=15, st=2, sr=future_ts
        ),
        _task_create(
            "G5cCWeV76KdYzuRFHpfU4X",
            "Water houseplants",
            ix=20,
            st=2,
            rr={"ft": 0, "ic": 1, "nt": 1},
        ),
        _task_create(
            "PUcyA7jg4UW6zksgnHeA7v",
            "Research insulation options",
            ix=25,
            st=2,
            pr=["HFiZWtutsAB7u9GeDToisD"],
        ),
    ]

    store = store_from_journal(journal)
    assert run_cli("someday", store) == get_fixture("someday_filtered")


def test_someday_detailed_shows_notes_and_checklist(store_from_journal) -> None:
    journal = [
        _task_create(
            "HFiZWtutsAB7u9GeDToisD",
            "Long-term writing goals",
            ix=5,
            st=2,
            tp=1,
            nt={"_t": "tx", "t": 1, "v": "Outline two themes"},
        ),
        _task_create(
            "LgYa4JKjhGtSboGa59LaAS",
            "Build a someday reading list",
            ix=10,
            st=2,
            nt={"_t": "tx", "t": 1, "v": "Focus on classics\nKeep it fun"},
        ),
        _checklist_create(
            "TNae8PMrKZe1b686DHtoh",
            "LgYa4JKjhGtSboGa59LaAS",
            "Pick five titles",
            ix=1,
            ss=0,
        ),
        _checklist_create(
            "KMV5DeN7swtQBu8hCYN5h4",
            "LgYa4JKjhGtSboGa59LaAS",
            "Borrow one book",
            ix=2,
            ss=3,
        ),
    ]

    store = store_from_journal(journal)
    assert run_cli("someday --detailed", store) == get_fixture("someday_detailed")
