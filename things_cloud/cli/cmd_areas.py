"""Areas list, new, and edit commands."""

import argparse
import sys
import time

from things_cloud.client import ThingsCloudClient
from things_cloud.ids import random_task_id
from things_cloud.store import ThingsStore
from things_cloud.schema import ENTITY_AREA
from things_cloud.cli.common import (
    BOLD,
    MAGENTA,
    GREEN,
    DIM,
    ICONS,
    CommandHandler,
    colored,
    _id_prefix,
    _resolve_tag_ids,
    _apply_tag_changes,
    _adapt_store_command,
    tag_edit_parent,
)


def cmd_areas(store: ThingsStore, args: argparse.Namespace) -> None:
    """Show all areas."""
    areas = store.areas()

    if not areas:
        print(colored("No areas.", DIM))
        return

    print(colored(f"{ICONS.area} Areas  ({len(areas)})", BOLD + MAGENTA))
    print()

    id_prefix_len = store.unique_prefix_length([area.uuid for area in areas])

    for area in areas:
        tags = ""
        if area.tags:
            tag_names = [store.resolve_tag_title(t) for t in area.tags]
            tags = colored("  [" + ", ".join(tag_names) + "]", DIM)
        print(
            f"  {_id_prefix(area.uuid, id_prefix_len)} "
            f"{colored(ICONS.area, DIM)} {area.title}{tags}"
        )


def cmd_new_area(
    store: ThingsStore, args: argparse.Namespace, client: ThingsCloudClient
) -> None:
    """Create a new area with just a title."""
    title = args.title.strip()
    if not title:
        print("Area title cannot be empty.", file=sys.stderr)
        return

    now_ts = time.time()
    props = {
        "tt": title,
        "ix": 0,
        "xx": {"_t": "oo", "sn": {}},
        "cd": now_ts,
        "md": now_ts,
    }

    if args.tags:
        tag_ids, tag_err = _resolve_tag_ids(store, args.tags)
        if tag_err:
            print(tag_err, file=sys.stderr)
            return
        props["tg"] = tag_ids

    new_uuid = random_task_id()
    try:
        client.create_task(new_uuid, props, entity=ENTITY_AREA)
    except Exception as e:
        print(f"Failed to create area: {e}", file=sys.stderr)
        return

    print(colored(f"{ICONS.done} Created", GREEN), f"{title}  {colored(new_uuid, DIM)}")


def cmd_edit_area(
    store: ThingsStore, args: argparse.Namespace, client: ThingsCloudClient
) -> None:
    """Edit an area: title only (no tag editing yet)."""
    area, err, ambiguous = store.resolve_area_identifier(args.area_id)
    if not area:
        print(err, file=sys.stderr)
        if ambiguous:
            id_prefix_len = store.unique_prefix_length([a.uuid for a in ambiguous])
            for match in ambiguous:
                print(
                    f"  {_id_prefix(match.uuid, id_prefix_len)} "
                    f"{colored(ICONS.area, DIM)} {match.title}"
                )
        return

    update: dict = {}
    labels: list[str] = []

    if args.title is not None:
        title = args.title.strip()
        if not title:
            print("Area title cannot be empty.", file=sys.stderr)
            return
        update["tt"] = title
        labels.append("title")

    add_tags_raw = getattr(args, "add_tags", None)
    remove_tags_raw = getattr(args, "remove_tags", None)
    if add_tags_raw or remove_tags_raw:
        new_tags, tag_labels, tag_err = _apply_tag_changes(
            area.tags, add_tags_raw, remove_tags_raw, store
        )
        if tag_err:
            print(tag_err, file=sys.stderr)
            return
        if new_tags is not None:
            update["tg"] = new_tags
            labels.extend(tag_labels)

    if not update:
        print("No edit changes requested.", file=sys.stderr)
        return

    try:
        client.update_task_fields(area.uuid, update, entity=ENTITY_AREA)
    except Exception as e:
        print(f"Failed to edit area: {e}", file=sys.stderr)
        return

    print(
        colored(f"{ICONS.done} Edited", GREEN),
        f"{(update.get('tt') or area.title)}  {colored(area.uuid, DIM)}",
        colored(f"({', '.join(labels)})", DIM),
    )


def register(subparsers) -> dict[str, CommandHandler]:
    areas_parser = subparsers.add_parser("areas", help="Show or create areas")
    areas_subs = areas_parser.add_subparsers(dest="areas_cmd", metavar="<subcommand>")
    areas_subs.add_parser("list", help="Show all areas")
    areas_new_parser = areas_subs.add_parser("new", help="Create a new area")
    areas_new_parser.add_argument("title", help="Area title")
    areas_new_parser.add_argument(
        "--tags",
        help="Comma-separated tags (titles or UUID prefixes)",
    )
    areas_edit_parser = areas_subs.add_parser(
        "edit",
        help="Edit an area title or tags",
        parents=[tag_edit_parent],
    )
    areas_edit_parser.add_argument(
        "area_id",
        help="Area UUID (or unique UUID prefix)",
    )
    areas_edit_parser.add_argument(
        "--title",
        help="Replace title",
    )
    # Make 'list' the default when no subcommand given
    areas_parser.set_defaults(areas_cmd="list")

    return {
        "areas": _adapt_store_command(cmd_areas),
        "areas:new": cmd_new_area,
        "areas:edit": cmd_edit_area,
    }
