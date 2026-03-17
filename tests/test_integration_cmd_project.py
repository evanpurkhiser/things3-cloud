from collections.abc import Callable

from things_cloud.store import ThingsStore

from tests.helpers import get_fixture, run_cli


def _task_create(
    uuid: str,
    title: str,
    *,
    ix: int,
    st: int = 1,
    ss: int = 0,
    tp: int = 0,
    pr: str | None = None,
    agr: str | None = None,
    nt: dict | None = None,
) -> dict:
    props: dict = {
        "tt": title,
        "tp": tp,
        "ss": ss,
        "st": st,
        "ix": ix,
        "cd": 1710000000,
        "md": 1710000000,
    }
    if pr is not None:
        props["pr"] = [pr]
    if agr is not None:
        props["agr"] = [agr]
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
            "p": {
                "tt": title,
                "ts": [task_uuid],
                "ss": ss,
                "ix": ix,
                "cd": 1710000000,
                "md": 1710000000,
            },
        }
    }


def test_project_not_found_has_no_stdout(
    store_from_journal: Callable[[list[dict]], ThingsStore],
) -> None:
    journal = [_task_create("a-project-0001", "Kitchen Refresh", ix=10, tp=1)]

    assert run_cli("project zzzzz", store_from_journal(journal)) == ""


def test_project_empty(
    store_from_journal: Callable[[list[dict]], ThingsStore],
) -> None:
    journal = [_task_create("a-project-0001", "Backyard Renovation", ix=10, tp=1)]

    assert run_cli("project a", store_from_journal(journal)) == get_fixture(
        "project_empty"
    )


def test_project_grouped_with_progress_counts(
    store_from_journal: Callable[[list[dict]], ThingsStore],
) -> None:
    journal = [
        _task_create("a-project-0001", "Release Plan", ix=10, tp=1),
        _task_create("b-task-0001", "Draft announcement", ix=10, pr="a-project-0001"),
        _task_create(
            "c-task-0001",
            "Publish release notes",
            ix=20,
            pr="a-project-0001",
            ss=3,
        ),
        _task_create(
            "d-heading-001",
            "QA",
            ix=100,
            tp=2,
            pr="a-project-0001",
        ),
        _task_create(
            "e-task-0001",
            "Run regression suite",
            ix=110,
            agr="d-heading-001",
        ),
        _task_create(
            "f-task-0001",
            "Capture screenshots",
            ix=120,
            agr="d-heading-001",
        ),
    ]

    assert run_cli("project a", store_from_journal(journal)) == get_fixture(
        "project_grouped"
    )


def test_project_detailed_with_notes_and_checklist(
    store_from_journal: Callable[[list[dict]], ThingsStore],
) -> None:
    journal = [
        _task_create("a-project-0001", "Conference Trip", ix=10, tp=1),
        _task_create(
            "b-task-0001",
            "Finalize packing",
            ix=10,
            pr="a-project-0001",
            nt={"_t": "tx", "t": 1, "v": "Bring carry-on only\nCharge battery pack"},
        ),
        _checklist_create("c-check-0001", "b-task-0001", "Passport", ix=10),
        _checklist_create("d-check-0001", "b-task-0001", "Headphones", ix=20, ss=3),
    ]

    assert run_cli("project a --detailed", store_from_journal(journal)) == get_fixture(
        "project_detailed"
    )
