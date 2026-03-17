from collections.abc import Callable

from things_cloud.store import ThingsStore

from tests.helpers import get_fixture, run_cli


def test_areas_empty(store_from_journal: Callable[[list[dict]], ThingsStore]) -> None:
    assert run_cli("areas", store_from_journal([])) == get_fixture("areas_empty")


def test_areas_basic_list_orders_by_index(
    store_from_journal: Callable[[list[dict]], ThingsStore],
) -> None:
    journal = [
        {
            "b-area-0002": {
                "t": 0,
                "e": "Area3",
                "p": {"tt": "Errands", "ix": 20},
            }
        },
        {
            "a-area-0001": {
                "t": 0,
                "e": "Area3",
                "p": {"tt": "Home", "ix": 10},
            }
        },
        {
            "c-area-0003": {
                "t": 0,
                "e": "Area3",
                "p": {"tt": "Work", "ix": 30},
            }
        },
    ]

    assert run_cli("areas", store_from_journal(journal)) == get_fixture(
        "areas_basic_ordering"
    )


def test_areas_renders_tag_titles(
    store_from_journal: Callable[[list[dict]], ThingsStore],
) -> None:
    journal = [
        {
            "a-area-0001": {
                "t": 0,
                "e": "Area3",
                "p": {"tt": "Home", "ix": 10, "tg": ["x-tag-0001", "y-tag-0002"]},
            }
        },
        {
            "x-tag-0001": {
                "t": 0,
                "e": "Tag4",
                "p": {"tt": "focus", "ix": 10},
            }
        },
        {
            "y-tag-0002": {
                "t": 0,
                "e": "Tag4",
                "p": {"tt": "chores", "ix": 20},
            }
        },
    ]

    assert run_cli("areas", store_from_journal(journal)) == get_fixture("areas_tags")


def test_areas_blank_title_tags_fall_back_to_tag_ids(
    store_from_journal: Callable[[list[dict]], ThingsStore],
) -> None:
    journal = [
        {
            "a-area-0001": {
                "t": 0,
                "e": "Area3",
                "p": {
                    "tt": "Ops",
                    "ix": 10,
                    "tg": ["x-tag-0001", "y-tag-blank-0002", "z-tag-space-0003"],
                },
            }
        },
        {
            "x-tag-0001": {
                "t": 0,
                "e": "Tag4",
                "p": {"tt": "active", "ix": 10},
            }
        },
        {
            "y-tag-blank-0002": {
                "t": 0,
                "e": "Tag4",
                "p": {"tt": "", "ix": 20},
            }
        },
        {
            "z-tag-space-0003": {
                "t": 0,
                "e": "Tag4",
                "p": {"tt": "   ", "ix": 30},
            }
        },
    ]

    assert run_cli("areas", store_from_journal(journal)) == get_fixture(
        "areas_blank_title_tags"
    )
