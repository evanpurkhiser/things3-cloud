from collections.abc import Callable

from things_cloud.store import ThingsStore

from tests.helpers import get_fixture, run_cli


def _area_create(
    uuid: str, title: str, *, ix: int, tags: list[str] | None = None
) -> dict:
    return {
        uuid: {
            "t": 0,
            "e": "Area3",
            "p": {"tt": title, "ix": ix, "tg": tags or []},
        }
    }


def _tag_create(uuid: str, title: str, *, ix: int) -> dict:
    return {uuid: {"t": 0, "e": "Tag4", "p": {"tt": title, "ix": ix}}}


def _task_create(
    uuid: str,
    title: str,
    *,
    ix: int,
    ss: int = 0,
    area_uuid: str | None = None,
    project_uuid: str | None = None,
    nt: dict | None = None,
) -> dict:
    props = {
        "tt": title,
        "tp": 0,
        "st": 1,
        "ss": ss,
        "ix": ix,
        "cd": 1,
        "md": 1,
    }
    if area_uuid:
        props["ar"] = [area_uuid]
    if project_uuid:
        props["pr"] = [project_uuid]
    if nt is not None:
        props["nt"] = nt
    return {uuid: {"t": 0, "e": "Task6", "p": props}}


def _project_create(
    uuid: str,
    title: str,
    *,
    ix: int,
    ss: int = 0,
    area_uuid: str | None = None,
    nt: dict | None = None,
) -> dict:
    props = {
        "tt": title,
        "tp": 1,
        "st": 1,
        "ss": ss,
        "ix": ix,
        "cd": 1,
        "md": 1,
    }
    if area_uuid:
        props["ar"] = [area_uuid]
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


def test_area_lists_loose_tasks_and_projects(
    store_from_journal: Callable[[list[dict]], ThingsStore],
) -> None:
    area_uuid = "a-area-0001"
    journal = [
        _area_create(area_uuid, "Home Ops", ix=10),
        _task_create("b-task-0001", "Replace air filter", ix=10, area_uuid=area_uuid),
        _task_create("c-task-0001", "Schedule plumber", ix=20, area_uuid=area_uuid),
        _project_create("d-proj-0001", "Kitchen Refresh", ix=30, area_uuid=area_uuid),
        _project_create("e-proj-0001", "Garage Cleanup", ix=40, area_uuid=area_uuid),
    ]

    assert run_cli(f"area {area_uuid}", store_from_journal(journal)) == get_fixture(
        "area_loose_projects"
    )


def test_area_all_includes_completed_tasks_and_projects(
    store_from_journal: Callable[[list[dict]], ThingsStore],
) -> None:
    area_uuid = "f-area-0001"
    journal = [
        _area_create(area_uuid, "Work", ix=10),
        _task_create("g-task-0001", "Draft roadmap", ix=10, area_uuid=area_uuid),
        _task_create(
            "h-task-0001",
            "Archive old docs",
            ix=20,
            ss=3,
            area_uuid=area_uuid,
        ),
        _project_create("i-proj-0001", "Q2 planning", ix=30, area_uuid=area_uuid),
        _project_create(
            "j-proj-0001",
            "Legacy migration",
            ix=40,
            ss=3,
            area_uuid=area_uuid,
        ),
    ]

    store = store_from_journal(journal)
    assert run_cli(f"area {area_uuid}", store) == get_fixture("area_all_default")
    assert run_cli(f"area {area_uuid} --all", store) == get_fixture(
        "area_all_with_completed"
    )


def test_area_header_shows_tags(
    store_from_journal: Callable[[list[dict]], ThingsStore],
) -> None:
    area_uuid = "k-area-0001"
    journal = [
        _tag_create("l-tag-0001", "Focus", ix=10),
        _tag_create("m-tag-0001", "Admin", ix=20),
        _area_create(area_uuid, "Work", ix=10, tags=["l-tag-0001", "m-tag-0001"]),
    ]

    assert run_cli(f"area {area_uuid}", store_from_journal(journal)) == get_fixture(
        "area_header_tags"
    )


def test_area_detailed_shows_task_and_project_notes_with_checklist(
    store_from_journal: Callable[[list[dict]], ThingsStore],
) -> None:
    area_uuid = "n-area-0001"
    journal = [
        _area_create(area_uuid, "Personal", ix=10),
        _task_create(
            "o-task-0001",
            "Plan weekend",
            ix=10,
            area_uuid=area_uuid,
            nt={"_t": "tx", "t": 1, "ch": 0, "v": "Book hotel\nPack bags"},
        ),
        _checklist_create("p-item-0001", "o-task-0001", "Charge camera", ix=10, ss=0),
        _checklist_create("q-item-0001", "o-task-0001", "Print tickets", ix=20, ss=3),
        _project_create(
            "r-proj-0001",
            "Spring cleaning",
            ix=20,
            area_uuid=area_uuid,
            nt={"_t": "tx", "t": 1, "ch": 0, "v": "Closet first\nThen garage"},
        ),
    ]

    assert run_cli(
        f"area {area_uuid} --detailed", store_from_journal(journal)
    ) == get_fixture("area_detailed")
