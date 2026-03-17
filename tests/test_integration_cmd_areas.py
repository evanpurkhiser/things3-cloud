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
            "DHWWGX9ZZsqznexxPvdUCG": {
                "t": 0,
                "e": "Area3",
                "p": {"tt": "Errands", "ix": 20},
            }
        },
        {
            "AeAbN2h6vFNJawRFx7mgdX": {
                "t": 0,
                "e": "Area3",
                "p": {"tt": "Home", "ix": 10},
            }
        },
        {
            "TiH5XjSXigme6pwMctAwjH": {
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
            "AeAbN2h6vFNJawRFx7mgdX": {
                "t": 0,
                "e": "Area3",
                "p": {
                    "tt": "Home",
                    "ix": 10,
                    "tg": ["JWsQXoB8VgrfRgYFBmz2x8", "XLVFi2whvAKGUQ6m32eGUF"],
                },
            }
        },
        {
            "JWsQXoB8VgrfRgYFBmz2x8": {
                "t": 0,
                "e": "Tag4",
                "p": {"tt": "focus", "ix": 10},
            }
        },
        {
            "XLVFi2whvAKGUQ6m32eGUF": {
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
            "HKQNxxbgQwcuHZkst36t3p": {
                "t": 0,
                "e": "Area3",
                "p": {
                    "tt": "Ops",
                    "ix": 10,
                    "tg": [
                        "MAcRZuZz4PYqRGUtxCGkRg",
                        "7fTdNw446YPi1bXzHtudjm",
                        "7sZ73dPdLCHdzLnyVZC3KF",
                    ],
                },
            }
        },
        {
            "MAcRZuZz4PYqRGUtxCGkRg": {
                "t": 0,
                "e": "Tag4",
                "p": {"tt": "active", "ix": 10},
            }
        },
        {
            "7fTdNw446YPi1bXzHtudjm": {
                "t": 0,
                "e": "Tag4",
                "p": {"tt": "", "ix": 20},
            }
        },
        {
            "7sZ73dPdLCHdzLnyVZC3KF": {
                "t": 0,
                "e": "Tag4",
                "p": {"tt": "   ", "ix": 30},
            }
        },
    ]

    assert run_cli("areas", store_from_journal(journal)) == get_fixture(
        "areas_blank_title_tags"
    )
