from tests.helpers import get_fixture, run_cli


def _area_create(uuid: str, title: str, *, ix: int) -> dict:
    return {uuid: {"t": 0, "e": "Area3", "p": {"tt": title, "ix": ix}}}


def _project_create(
    uuid: str,
    title: str,
    *,
    ix: int,
    st: int = 1,
    ss: int = 0,
    ar: list[str] | None = None,
    nt: str | dict | None = None,
) -> dict:
    props = {
        "tt": title,
        "tp": 1,
        "st": st,
        "ss": ss,
        "ix": ix,
        "cd": 1,
        "md": 1,
    }
    if ar is not None:
        props["ar"] = ar
    if nt is not None:
        props["nt"] = nt
    return {uuid: {"t": 0, "e": "Task6", "p": props}}


def _task_create(
    uuid: str,
    title: str,
    *,
    ix: int,
    pr: list[str],
    ss: int = 0,
) -> dict:
    return {
        uuid: {
            "t": 0,
            "e": "Task6",
            "p": {
                "tt": title,
                "st": 1,
                "ss": ss,
                "ix": ix,
                "pr": pr,
                "cd": 1,
                "md": 1,
            },
        }
    }


def test_projects_empty(store_from_journal) -> None:
    assert run_cli("projects", store_from_journal([])) == get_fixture("projects_empty")


def test_projects_basic_grouped_by_area(store_from_journal) -> None:
    journal = [
        _area_create("a-area-home-0001", "Home", ix=10),
        _area_create("b-area-work-0001", "Work", ix=20),
        _project_create("c-proj-loose-0001", "Unsorted Project", ix=5),
        _project_create(
            "d-proj-home-0001",
            "Kitchen Refresh",
            ix=10,
            ar=["a-area-home-0001"],
        ),
        _project_create(
            "e-proj-work-0001", "Q2 Planning", ix=20, ar=["b-area-work-0001"]
        ),
    ]

    assert run_cli("projects", store_from_journal(journal)) == get_fixture(
        "projects_basic_grouped"
    )


def test_projects_progress_markers_from_child_tasks(store_from_journal) -> None:
    journal = [
        _project_create("a-proj-alpha-0001", "Alpha Project", ix=10),
        _project_create("b-proj-beta-0001", "Beta Project", ix=20),
        _project_create("c-proj-gamma-0001", "Gamma Project", ix=30),
        _task_create(
            "d-task-a-0001", "Open item", ix=11, pr=["a-proj-alpha-0001"], ss=0
        ),
        _task_create(
            "e-task-b-0001", "Done item", ix=12, pr=["a-proj-alpha-0001"], ss=3
        ),
        _task_create(
            "f-task-c-0001", "Done item 2", ix=21, pr=["b-proj-beta-0001"], ss=3
        ),
    ]

    assert run_cli("projects", store_from_journal(journal)) == get_fixture(
        "projects_progress_markers"
    )


def test_projects_detailed_shows_project_notes(store_from_journal) -> None:
    journal = [
        _area_create("a-area-notes-0001", "Personal", ix=10),
        _project_create(
            "b-proj-notes-0001",
            "Trip Planning",
            ix=10,
            ar=["a-area-notes-0001"],
            nt={"_t": "tx", "t": 1, "v": "Book flights\nRenew passport"},
        ),
        _project_create(
            "c-proj-note2-0001",
            "Reading List",
            ix=20,
            nt={"_t": "tx", "t": 1, "v": "Pick 3 biographies"},
        ),
    ]

    assert run_cli("projects --detailed", store_from_journal(journal)) == get_fixture(
        "projects_detailed"
    )
