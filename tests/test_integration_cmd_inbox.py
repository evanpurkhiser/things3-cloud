from collections.abc import Callable

from things_cloud.store import ThingsStore

from tests.helpers import get_fixture, run_cli


def test_inbox_empty(store_from_journal: Callable[[list[dict]], ThingsStore]) -> None:
    assert run_cli("inbox", store_from_journal([])) == get_fixture("inbox_empty")


def test_inbox_basic_list(
    store_from_journal: Callable[[list[dict]], ThingsStore],
) -> None:
    journal = [
        {
            "task-inbox-alpha-0001": {
                "t": 0,
                "e": "Task6",
                "p": {
                    "tt": "Buy oat milk",
                    "st": 0,
                    "ss": 0,
                    "ix": 10,
                    "cd": 1710000000,
                    "md": 1710000000,
                },
            }
        },
        {
            "task-inbox-beta-0002": {
                "t": 0,
                "e": "Task6",
                "p": {
                    "tt": "Email landlord",
                    "st": 0,
                    "ss": 0,
                    "ix": 20,
                    "cd": 1710000100,
                    "md": 1710000100,
                },
            }
        },
    ]

    assert run_cli("inbox", store_from_journal(journal)) == get_fixture("inbox_basic")


def test_inbox_ignores_project_and_area_scoped_tasks(
    store_from_journal: Callable[[list[dict]], ThingsStore],
) -> None:
    journal = [
        {
            "task-loose-0001": {
                "t": 0,
                "e": "Task6",
                "p": {
                    "tt": "Top-level inbox task",
                    "st": 0,
                    "ss": 0,
                    "ix": 10,
                    "cd": 1710000000,
                    "md": 1710000000,
                },
            }
        },
        {
            "project-home-0001": {
                "t": 0,
                "e": "Task6",
                "p": {
                    "tt": "Kitchen Refresh",
                    "tp": 1,
                    "st": 1,
                    "ss": 0,
                    "ix": 100,
                    "cd": 1710000000,
                    "md": 1710000000,
                },
            }
        },
        {
            "task-proj-0001": {
                "t": 0,
                "e": "Task6",
                "p": {
                    "tt": "Choose tile samples",
                    "st": 0,
                    "ss": 0,
                    "pr": ["project-home-0001"],
                    "ix": 110,
                    "cd": 1710000200,
                    "md": 1710000200,
                },
            }
        },
        {
            "area-work-0001": {
                "t": 0,
                "e": "Area3",
                "p": {"tt": "Work", "ix": 200},
            }
        },
        {
            "task-area-0001": {
                "t": 0,
                "e": "Task6",
                "p": {
                    "tt": "Draft quarterly goals",
                    "st": 0,
                    "ss": 0,
                    "ar": ["area-work-0001"],
                    "ix": 210,
                    "cd": 1710000300,
                    "md": 1710000300,
                },
            }
        },
        {
            "project-area-0001": {
                "t": 0,
                "e": "Task6",
                "p": {
                    "tt": "Migration Plan",
                    "tp": 1,
                    "st": 1,
                    "ss": 0,
                    "ar": ["area-work-0001"],
                    "ix": 220,
                    "cd": 1710000400,
                    "md": 1710000400,
                },
            }
        },
        {
            "task-area-proj-1": {
                "t": 0,
                "e": "Task6",
                "p": {
                    "tt": "Write rollout checklist",
                    "st": 0,
                    "ss": 0,
                    "pr": ["project-area-0001"],
                    "ix": 230,
                    "cd": 1710000500,
                    "md": 1710000500,
                },
            }
        },
    ]

    assert run_cli("inbox", store_from_journal(journal)) == get_fixture(
        "inbox_scoped_filtered"
    )


def test_inbox_detailed_mode_shows_notes_and_checklist(
    store_from_journal: Callable[[list[dict]], ThingsStore],
) -> None:
    journal = [
        {
            "task-detail-0001": {
                "t": 0,
                "e": "Task6",
                "p": {
                    "tt": "Plan weekend trip",
                    "st": 0,
                    "ss": 0,
                    "ix": 10,
                    "cd": 1710000000,
                    "md": 1710000000,
                    "nt": {
                        "_t": "tx",
                        "t": 1,
                        "ch": 0,
                        "v": "Book train\nPack light",
                    },
                },
            }
        },
        {
            "task-check-0001": {
                "t": 0,
                "e": "Task6",
                "p": {
                    "tt": "Grocery prep",
                    "st": 0,
                    "ss": 0,
                    "ix": 20,
                    "cd": 1710000100,
                    "md": 1710000100,
                    "nt": {
                        "_t": "tx",
                        "t": 1,
                        "ch": 0,
                        "v": "For taco night",
                    },
                },
            }
        },
        {
            "check-item-a001": {
                "t": 0,
                "e": "ChecklistItem3",
                "p": {
                    "tt": "Tortillas",
                    "ss": 0,
                    "ts": ["task-check-0001"],
                    "ix": 10,
                    "cd": 1710000200,
                    "md": 1710000200,
                },
            }
        },
        {
            "check-item-b002": {
                "t": 0,
                "e": "ChecklistItem3",
                "p": {
                    "tt": "Salsa",
                    "ss": 3,
                    "ts": ["task-check-0001"],
                    "ix": 20,
                    "cd": 1710000201,
                    "md": 1710000201,
                },
            }
        },
    ]

    assert run_cli("inbox --detailed", store_from_journal(journal)) == get_fixture(
        "inbox_detailed"
    )
