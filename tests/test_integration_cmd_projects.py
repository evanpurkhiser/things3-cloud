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
        _area_create("Lrfk3xS36P7vBxJyuKBAfk", "Home", ix=10),
        _area_create("AhSY1yLqv7zt7uYCeSXZka", "Work", ix=20),
        _project_create("PunE7qFpjY3FcMP8SytB68", "Unsorted Project", ix=5),
        _project_create(
            "GG5a8qj6uFq5WtqbxywYng",
            "Kitchen Refresh",
            ix=10,
            ar=["Lrfk3xS36P7vBxJyuKBAfk"],
        ),
        _project_create(
            "Nx1QfHaTQ9e5C7MzKt84rd",
            "Q2 Planning",
            ix=20,
            ar=["AhSY1yLqv7zt7uYCeSXZka"],
        ),
    ]

    assert run_cli("projects", store_from_journal(journal)) == get_fixture(
        "projects_basic_grouped"
    )


def test_projects_progress_markers_from_child_tasks(store_from_journal) -> None:
    journal = [
        _project_create("5XZBUVvMJJ3xafxKzzeRrQ", "Alpha Project", ix=10),
        _project_create("4WpRGbNLXBaZNNNXqxgom3", "Beta Project", ix=20),
        _project_create("9wTxE2QkkXB4Enka84kC2X", "Gamma Project", ix=30),
        _task_create(
            "VCB1NnksKA32TUamkQtkaH",
            "Open item",
            ix=11,
            pr=["5XZBUVvMJJ3xafxKzzeRrQ"],
            ss=0,
        ),
        _task_create(
            "KDYKBhM74s38f9aXVdGgTj",
            "Done item",
            ix=12,
            pr=["5XZBUVvMJJ3xafxKzzeRrQ"],
            ss=3,
        ),
        _task_create(
            "KPCXPFFjKzEtaqkv4sAHYJ",
            "Done item 2",
            ix=21,
            pr=["4WpRGbNLXBaZNNNXqxgom3"],
            ss=3,
        ),
    ]

    assert run_cli("projects", store_from_journal(journal)) == get_fixture(
        "projects_progress_markers"
    )


def test_projects_detailed_shows_project_notes(store_from_journal) -> None:
    journal = [
        _area_create("NNKhZuXngWNVsiZ9xMoeDK", "Personal", ix=10),
        _project_create(
            "KdfyiRxGVe3QFBLuT9Y9uD",
            "Trip Planning",
            ix=10,
            ar=["NNKhZuXngWNVsiZ9xMoeDK"],
            nt={"_t": "tx", "t": 1, "v": "Book flights\nRenew passport"},
        ),
        _project_create(
            "5kjDzoGubps33DLVC9ptR6",
            "Reading List",
            ix=20,
            nt={"_t": "tx", "t": 1, "v": "Pick 3 biographies"},
        ),
    ]

    assert run_cli("projects --detailed", store_from_journal(journal)) == get_fixture(
        "projects_detailed"
    )
