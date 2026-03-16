import argparse
import io
import unittest
from contextlib import redirect_stderr, redirect_stdout
from typing import Any, cast

import cli
from things_cloud.store import ThingsStore


def _schedule_state() -> dict[str, dict]:
    return {
        "task-schedule-000": {
            "e": "Task6",
            "p": {"tt": "Schedule me", "ss": 0, "st": 0, "ix": 1},
        }
    }


class _FakeClient:
    def __init__(self) -> None:
        self.calls: list[tuple[str, dict, str]] = []

    def update_task_fields(
        self, task_uuid: str, props: dict, entity: str = "Task6"
    ) -> int:
        self.calls.append((task_uuid, props, entity))
        return 1


class CmdScheduleTests(unittest.TestCase):
    def setUp(self) -> None:
        self.store = ThingsStore(_schedule_state())
        self.client = _FakeClient()

    def test_schedule_today_updates_start_fields(self) -> None:
        args = argparse.Namespace(
            task_id="task-schedule",
            when="today",
            deadline_date=None,
            clear_deadline=False,
        )

        out = io.StringIO()
        err = io.StringIO()
        with redirect_stdout(out), redirect_stderr(err):
            cli.cmd_schedule(self.store, args, cast(Any, self.client))

        self.assertEqual(err.getvalue(), "")
        self.assertEqual(len(self.client.calls), 1)
        task_uuid, props, entity = self.client.calls[0]
        self.assertEqual(task_uuid, "task-schedule-000")
        self.assertEqual(entity, "Task6")
        self.assertEqual(props["st"], 1)
        self.assertEqual(props["sb"], 0)
        self.assertIn("sr", props)

    def test_schedule_can_set_deadline_without_when_change(self) -> None:
        args = argparse.Namespace(
            task_id="task-schedule",
            when=None,
            deadline_date="2026-04-10",
            clear_deadline=False,
        )

        out = io.StringIO()
        err = io.StringIO()
        with redirect_stdout(out), redirect_stderr(err):
            cli.cmd_schedule(self.store, args, cast(Any, self.client))

        self.assertEqual(err.getvalue(), "")
        self.assertEqual(len(self.client.calls), 1)
        _task_uuid, props, _entity = self.client.calls[0]
        self.assertIn("dd", props)
        self.assertNotIn("st", props)


if __name__ == "__main__":
    unittest.main()
