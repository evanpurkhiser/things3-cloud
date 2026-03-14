import argparse
import io
import unittest
from contextlib import redirect_stderr, redirect_stdout
from datetime import datetime, timezone
from typing import Any, cast

import cli
from things_cloud.store import ThingsStore


class _FakeClient:
    def __init__(self) -> None:
        self.calls: list[tuple[str, dict, str]] = []

    def update_task_fields(
        self, task_uuid: str, props: dict, entity: str = "Task6"
    ) -> int:
        self.calls.append((task_uuid, props, entity))
        return 1


def _today_ts() -> int:
    return int(
        datetime.now(tz=timezone.utc)
        .replace(hour=0, minute=0, second=0, microsecond=0)
        .timestamp()
    )


class CmdReorderTests(unittest.TestCase):
    def test_reorder_inbox_updates_ix_only(self) -> None:
        state = {
            "task-item-000": {
                "e": "Task6",
                "p": {"tt": "Item", "ss": 0, "st": 0, "ix": 1},
            },
            "task-anchor-0": {
                "e": "Task6",
                "p": {"tt": "Anchor", "ss": 0, "st": 0, "ix": 10},
            },
        }
        store = ThingsStore(state)
        client = _FakeClient()
        args = argparse.Namespace(
            item_id="task-item", before_id=None, after_id="task-anchor"
        )

        out = io.StringIO()
        err = io.StringIO()
        with redirect_stdout(out), redirect_stderr(err):
            cli.cmd_reorder(store, args, cast(Any, client))

        self.assertEqual(err.getvalue(), "")
        self.assertEqual(len(client.calls), 1)
        task_uuid, props, entity = client.calls[0]
        self.assertEqual(task_uuid, "task-item-000")
        self.assertEqual(entity, "Task6")
        self.assertEqual(props, {"ix": 11})

    def test_reorder_today_updates_today_fields(self) -> None:
        day_ts = _today_ts()
        state = {
            "task-item-000": {
                "e": "Task6",
                "p": {
                    "tt": "Item",
                    "ss": 0,
                    "st": 1,
                    "sr": day_ts,
                    "tir": day_ts,
                    "ti": 50,
                    "sb": 0,
                    "ix": 1,
                },
            },
            "task-anchor-0": {
                "e": "Task6",
                "p": {
                    "tt": "Anchor",
                    "ss": 0,
                    "st": 1,
                    "sr": day_ts,
                    "tir": day_ts,
                    "ti": 200,
                    "sb": 1,
                    "ix": 100,
                },
            },
        }
        store = ThingsStore(state)
        client = _FakeClient()
        args = argparse.Namespace(
            item_id="task-item", before_id=None, after_id="task-anchor"
        )

        out = io.StringIO()
        err = io.StringIO()
        with redirect_stdout(out), redirect_stderr(err):
            cli.cmd_reorder(store, args, cast(Any, client))

        self.assertEqual(err.getvalue(), "")
        self.assertEqual(len(client.calls), 1)
        task_uuid, props, entity = client.calls[0]
        self.assertEqual(task_uuid, "task-item-000")
        self.assertEqual(entity, "Task6")
        self.assertEqual(props.get("tir"), day_ts)
        self.assertEqual(props.get("ti"), 201)
        self.assertEqual(props.get("sb"), 1)
        self.assertNotIn("ix", props)

    def test_reorder_inbox_rebalances_when_no_index_gap(self) -> None:
        state = {
            "task-item-000": {
                "e": "Task6",
                "p": {"tt": "Item", "ss": 0, "st": 0, "ix": 5},
            },
            "task-anchor-0": {
                "e": "Task6",
                "p": {"tt": "Anchor", "ss": 0, "st": 0, "ix": 100},
            },
            "task-next-000": {
                "e": "Task6",
                "p": {"tt": "Next", "ss": 0, "st": 0, "ix": 101},
            },
        }
        store = ThingsStore(state)
        client = _FakeClient()
        args = argparse.Namespace(
            item_id="task-item", before_id=None, after_id="task-anchor"
        )

        out = io.StringIO()
        err = io.StringIO()
        with redirect_stdout(out), redirect_stderr(err):
            cli.cmd_reorder(store, args, cast(Any, client))

        self.assertEqual(err.getvalue(), "")
        self.assertGreaterEqual(len(client.calls), 2)

        updates = {task_uuid: props["ix"] for task_uuid, props, _entity in client.calls}
        self.assertIn("task-item-000", updates)
        self.assertIn("task-anchor-0", updates)
        self.assertIn("task-next-000", updates)
        self.assertGreater(updates["task-item-000"], updates["task-anchor-0"])
        self.assertLess(updates["task-item-000"], updates["task-next-000"])


if __name__ == "__main__":
    unittest.main()
