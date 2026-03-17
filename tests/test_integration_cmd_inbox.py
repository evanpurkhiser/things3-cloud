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
            "XnDJNLT4hkk4hXBGLkoVH": {
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
            "ESZrdo2KmGjgznpPCHtBA7": {
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
            "NtDfRHDHdvGke5HdLrXBTi": {
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
            "MFF5U5YwqoP2yoMC4DJ3Vc": {
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
            "DrtEZ6JNAkpZ6XnE7nkMM9": {
                "t": 0,
                "e": "Task6",
                "p": {
                    "tt": "Choose tile samples",
                    "st": 0,
                    "ss": 0,
                    "pr": ["MFF5U5YwqoP2yoMC4DJ3Vc"],
                    "ix": 110,
                    "cd": 1710000200,
                    "md": 1710000200,
                },
            }
        },
        {
            "K1Hx4pgPjYDocEx695rSeM": {
                "t": 0,
                "e": "Area3",
                "p": {"tt": "Work", "ix": 200},
            }
        },
        {
            "QNKTu3e1HXKNLZ6KW7RRQD": {
                "t": 0,
                "e": "Task6",
                "p": {
                    "tt": "Draft quarterly goals",
                    "st": 0,
                    "ss": 0,
                    "ar": ["K1Hx4pgPjYDocEx695rSeM"],
                    "ix": 210,
                    "cd": 1710000300,
                    "md": 1710000300,
                },
            }
        },
        {
            "Lm87wGotuCnt2FbkAguBbV": {
                "t": 0,
                "e": "Task6",
                "p": {
                    "tt": "Migration Plan",
                    "tp": 1,
                    "st": 1,
                    "ss": 0,
                    "ar": ["K1Hx4pgPjYDocEx695rSeM"],
                    "ix": 220,
                    "cd": 1710000400,
                    "md": 1710000400,
                },
            }
        },
        {
            "76SUcghw2hNShXDWdjofDT": {
                "t": 0,
                "e": "Task6",
                "p": {
                    "tt": "Write rollout checklist",
                    "st": 0,
                    "ss": 0,
                    "pr": ["Lm87wGotuCnt2FbkAguBbV"],
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
            "TgKcfPUkgE7AfbxFZzDYcg": {
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
            "59oSjLnK37e4ADDF7aiQ6e": {
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
            "BhW6Mf5sE4q7eziXVPDKqF": {
                "t": 0,
                "e": "ChecklistItem3",
                "p": {
                    "tt": "Tortillas",
                    "ss": 0,
                    "ts": ["59oSjLnK37e4ADDF7aiQ6e"],
                    "ix": 10,
                    "cd": 1710000200,
                    "md": 1710000200,
                },
            }
        },
        {
            "LGhsdBAgViHjcSQhmRGL7U": {
                "t": 0,
                "e": "ChecklistItem3",
                "p": {
                    "tt": "Salsa",
                    "ss": 3,
                    "ts": ["59oSjLnK37e4ADDF7aiQ6e"],
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
