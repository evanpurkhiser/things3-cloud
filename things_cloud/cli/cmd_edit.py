"""Edit task command."""

import argparse
import sys

from things_cloud.client import ThingsCloudClient
from things_cloud.store import ThingsStore
from things_cloud.schema import TaskStart
from things_cloud.cli.common import (
    GREEN,
    DIM,
    ICONS,
    CommandHandler,
    colored,
    fmt_resolve_error,
    _task6_note,
)


def cmd_edit(
    store: ThingsStore, args: argparse.Namespace, client: ThingsCloudClient
) -> None:
    """Edit one task: title, container, and notes."""
    task, err, ambiguous = store.resolve_mark_identifier(args.task_id)
    if not task:
        fmt_resolve_error(err, ambiguous, store)
        return

    if task.is_project:
        print(
            "Use 'projects edit' to edit a project.",
            file=sys.stderr,
        )
        return

    update: dict = {}
    labels: list[str] = []

    if args.title is not None:
        title = args.title.strip()
        if not title:
            print("Task title cannot be empty.", file=sys.stderr)
            return
        update["tt"] = title
        labels.append("title")

    if args.notes is not None:
        update["nt"] = (
            _task6_note(args.notes)
            if args.notes
            else {"_t": "tx", "t": 1, "ch": 0, "v": ""}
        )
        labels.append("notes")

    move_raw = (args.move_target or "").strip()
    if move_raw:
        move_l = move_raw.lower()
        if move_l == "inbox":
            update.update(
                {
                    "pr": [],
                    "ar": [],
                    "agr": [],
                    "st": TaskStart.INBOX,
                    "sr": None,
                    "tir": None,
                    "sb": 0,
                }
            )
            labels.append("move=inbox")
        elif move_l == "clear":
            update.update({"pr": [], "ar": [], "agr": []})
            if task.start == TaskStart.INBOX:
                update["st"] = TaskStart.ANYTIME
            labels.append("move=clear")
        else:
            project, _perr, _pamb = store.resolve_mark_identifier(move_raw)
            area, _aerr, _aamb = store.resolve_area_identifier(move_raw)

            project_uuid = project.uuid if project and project.is_project else None
            area_uuid = area.uuid if area else None

            if project_uuid and area_uuid:
                print(
                    f"Ambiguous --move target '{move_raw}' (matches project and area).",
                    file=sys.stderr,
                )
                return
            if project and not project.is_project:
                print(
                    "--move target must be Inbox, clear, a project ID, or an area ID.",
                    file=sys.stderr,
                )
                return
            if project_uuid:
                update.update({"pr": [project_uuid], "ar": [], "agr": []})
                if task.start == TaskStart.INBOX:
                    update["st"] = TaskStart.ANYTIME
                labels.append(f"move={move_raw}")
            elif area_uuid:
                update.update({"ar": [area_uuid], "pr": [], "agr": []})
                if task.start == TaskStart.INBOX:
                    update["st"] = TaskStart.ANYTIME
                labels.append(f"move={move_raw}")
            else:
                print(f"Container not found: {move_raw}", file=sys.stderr)
                return

    if not update:
        print("No edit changes requested.", file=sys.stderr)
        return

    try:
        client.update_task_fields(task.uuid, update, entity=task.entity)
    except Exception as e:
        print(f"Failed to edit item: {e}", file=sys.stderr)
        return

    print(
        colored(f"{ICONS.done} Edited", GREEN),
        f"{(update.get('tt') or task.title)}  {colored(task.uuid, DIM)}",
        colored(f"({', '.join(labels)})", DIM),
    )


def register(subparsers) -> dict[str, CommandHandler]:
    edit_parser = subparsers.add_parser(
        "edit", help="Edit a task title, container, or notes"
    )
    edit_parser.add_argument(
        "task_id",
        help="Task UUID (or unique UUID prefix)",
    )
    edit_parser.add_argument(
        "--title",
        help="Replace title",
    )
    edit_parser.add_argument(
        "--move",
        dest="move_target",
        help="Move to Inbox, clear, project UUID/prefix, or area UUID/prefix",
    )
    edit_parser.add_argument(
        "--notes",
        help="Replace notes (use empty string to clear)",
    )

    return {"edit": cmd_edit}
