from datetime import datetime, timezone

from things_cloud.store import ThingsStore

from tests.helpers import build_store_from_journal, get_fixture, run_cli


def _day_ts() -> int:
    return int(
        datetime.now(tz=timezone.utc)
        .replace(hour=0, minute=0, second=0, microsecond=0)
        .timestamp()
    )


def test_build_store_from_journal_applies_create_update_delete() -> None:
    journal = [
        {
            "task-1": {
                "t": 0,
                "e": "Task6",
                "p": {"tt": "Original", "ss": 0, "st": 0, "ix": 1, "cd": 1},
            }
        },
        {"task-1": {"t": 1, "e": "Task6", "p": {"tt": "Updated"}}},
        {
            "task-2": {
                "t": 0,
                "e": "Task6",
                "p": {"tt": "Delete me", "ss": 0, "st": 0, "ix": 2, "cd": 1},
            }
        },
        {"task-2": {"t": 2, "e": "Task6", "p": {}}},
    ]

    store = build_store_from_journal(journal)

    task = store.get_task("task-1")
    assert task is not None
    assert task.title == "Updated"
    assert store.get_task("task-2") is None


def test_run_cli_and_get_fixture() -> None:
    day_ts = _day_ts()
    store = ThingsStore(
        {
            "inbox-task-1": {
                "e": "Task6",
                "p": {
                    "tt": "Inbox task",
                    "ss": 0,
                    "st": 0,
                    "ix": 1,
                    "cd": 1,
                    "md": 1,
                    "sr": day_ts,
                },
            }
        }
    )

    assert run_cli("inbox", store) == get_fixture("integration_helpers_inbox")
