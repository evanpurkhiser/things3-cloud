from collections.abc import Callable

from things_cloud.store import ThingsStore

from tests.helpers import get_fixture, run_cli
from tests.mutating_fixtures import area, store
from tests.mutating_http_helpers import (
    assert_commit_payloads,
    assert_no_commits,
    p,
    run_cli_mutating_http,
)

NOW = 1_700_000_222.0
AREA_UUID = "MpkEei6ybkFS2n6SXvwfLf"


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


def test_areas_edit_title_payload() -> None:
    result = run_cli_mutating_http(
        f'areas edit {AREA_UUID} --title "New Name"',
        store(area(AREA_UUID, "Old Name")),
        extra_patches=[p("things_cloud.client.time.time", return_value=NOW)],
    )
    assert_commit_payloads(
        result,
        {AREA_UUID: {"t": 1, "e": "Area3", "p": {"tt": "New Name", "md": NOW}}},
    )


def test_areas_edit_no_changes_is_rejected() -> None:
    result = run_cli_mutating_http(
        f"areas edit {AREA_UUID}",
        store(area(AREA_UUID, "Home")),
    )
    assert_no_commits(result)
    assert result.stderr == "No edit changes requested.\n"


def test_areas_edit_empty_title_is_rejected() -> None:
    result = run_cli_mutating_http(
        f"areas edit {AREA_UUID} --title ''",
        store(area(AREA_UUID, "Home")),
    )
    assert_no_commits(result)
    assert result.stderr == "Area title cannot be empty.\n"
