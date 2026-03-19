"""Find command — search and filter tasks across the entire store."""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from typing import Optional

from things_cloud.store import (
    ThingsStore,
    Task,
    STATUS_INCOMPLETE,
    STATUS_COMPLETED,
    STATUS_CANCELED,
)
from things_cloud.cli.common import (
    BOLD,
    CYAN,
    DIM,
    ICONS,
    CommandHandler,
    colored,
    detailed_parent,
    fmt_task_line,
    fmt_task_with_note,
    fmt_project_line,
    fmt_project_with_note,
    _adapt_store_command,
    _resolve_single_tag,
)

ENTITY_TASK = "Task6"

# ---------------------------------------------------------------------------
# Date expression parsing
# ---------------------------------------------------------------------------

_OP_RE = re.compile(r"^(>=|<=|>|<|=)(.+)$")


def _today_utc() -> datetime:
    return datetime.now(tz=timezone.utc).replace(
        hour=0, minute=0, second=0, microsecond=0
    )


def _parse_date_value(value: str, flag: str) -> datetime:
    """Parse a date keyword or ISO date string into a UTC midnight datetime."""
    v = value.strip().lower()
    today = _today_utc()
    if v == "today":
        return today
    if v == "tomorrow":
        return today + timedelta(days=1)
    if v == "yesterday":
        return today - timedelta(days=1)
    try:
        dt = datetime.strptime(v, "%Y-%m-%d")
        return dt.replace(tzinfo=timezone.utc)
    except ValueError:
        raise ValueError(
            f"Invalid date for {flag}: {value!r}. "
            "Expected YYYY-MM-DD, 'today', 'tomorrow', or 'yesterday'."
        )


def _parse_date_expr(raw: str, flag: str) -> tuple[str, datetime]:
    """Parse '<op><date>' string. Returns (op, datetime)."""
    m = _OP_RE.match(raw.strip())
    if not m:
        raise ValueError(
            f"Invalid date expression for {flag}: {raw!r}. "
            "Expected an operator prefix: >, <, >=, <=, or =  (e.g. '<=2026-03-31')"
        )
    op, date_str = m.group(1), m.group(2)
    dt = _parse_date_value(date_str, flag)
    return op, dt


def _date_matches(field_dt: Optional[datetime], op: str, threshold: datetime) -> bool:
    """Compare a UTC-midnight field datetime against a threshold using op."""
    if field_dt is None:
        return False
    # Normalise both to UTC midnight for day-level comparison
    field_day = field_dt.astimezone(timezone.utc).replace(
        hour=0, minute=0, second=0, microsecond=0
    )
    threshold_day = threshold.replace(hour=0, minute=0, second=0, microsecond=0)
    if op == ">":
        return field_day > threshold_day
    if op == "<":
        return field_day < threshold_day
    if op == ">=":
        return field_day >= threshold_day
    if op == "<=":
        return field_day <= threshold_day
    if op == "=":
        return field_day == threshold_day
    return False


# ---------------------------------------------------------------------------
# Match result
# ---------------------------------------------------------------------------


@dataclass
class MatchResult:
    matched: bool
    checklist_only: bool = False  # True when query matched only via checklist items

    @staticmethod
    def no() -> "MatchResult":
        return MatchResult(matched=False)

    @staticmethod
    def yes(*, checklist_only: bool = False) -> "MatchResult":
        return MatchResult(matched=True, checklist_only=checklist_only)


# ---------------------------------------------------------------------------
# Filter logic
# ---------------------------------------------------------------------------


def _build_status_set(args: argparse.Namespace) -> Optional[set[int]]:
    """Return the set of allowed task statuses, or None to mean 'any'."""
    if getattr(args, "any_status", False):
        return None  # no restriction

    flags = {
        "incomplete": STATUS_INCOMPLETE,
        "completed": STATUS_COMPLETED,
        "canceled": STATUS_CANCELED,
    }
    chosen = {v for k, v in flags.items() if getattr(args, k, False)}

    # --completed-on implies completed status when no explicit status flag is set
    if not chosen and getattr(args, "completed_on", None):
        return {STATUS_COMPLETED}

    if not chosen:
        # Default: incomplete only (consistent with all other view commands)
        return {STATUS_INCOMPLETE}
    return chosen


def _matches(task: Task, store: ThingsStore, args: argparse.Namespace) -> MatchResult:
    """Return a MatchResult indicating whether the task passes all active filters."""

    # Skip headings always
    if task.is_heading:
        return MatchResult.no()

    # Skip trashed
    if task.trashed:
        return MatchResult.no()

    # Only Task6 entities (same as every other view)
    if task.entity != ENTITY_TASK:
        return MatchResult.no()

    # Status
    allowed_statuses = _build_status_set(args)
    if allowed_statuses is not None and task.status not in allowed_statuses:
        return MatchResult.no()

    # Title / notes / checklist query (case-insensitive substring)
    query = getattr(args, "query", None)
    checklist_only = False
    if query:
        q = query.lower()
        title_match = q in task.title.lower()
        notes_match = (
            bool(task.notes)
            and getattr(args, "notes", False)
            and q in task.notes.lower()
        )
        checklist_match = getattr(args, "checklists", False) and any(
            q in item.title.lower() for item in task.checklist_items
        )
        if not title_match and not notes_match and not checklist_match:
            return MatchResult.no()
        checklist_only = checklist_match and not title_match and not notes_match

    # Tag filter — multiple values are OR'd (task must match at least one)
    tag_filters: list[str] = getattr(args, "tag", None) or []
    if tag_filters:
        # Tag UUIDs are pre-resolved by cmd_find before _matches is called
        required_uuids: list[str] = getattr(args, "_resolved_tag_uuids", [])
        if not any(uuid in task.tags for uuid in required_uuids):
            return MatchResult.no()

    # Project filter — multiple values are OR'd
    project_filters: list[str] = getattr(args, "project", None) or []
    if project_filters:
        effective = store.effective_project_uuid(task)
        if not effective:
            return MatchResult.no()
        proj_task = store.get_task(effective)
        if proj_task is None:
            return MatchResult.no()
        if not any(
            pf.lower() in proj_task.title.lower() or effective.startswith(pf)
            for pf in project_filters
        ):
            return MatchResult.no()

    # Area filter — multiple values are OR'd
    area_filters: list[str] = getattr(args, "area", None) or []
    if area_filters:
        effective_area = store.effective_area_uuid(task)
        if not effective_area:
            return MatchResult.no()
        area_obj = store.get_area(effective_area)
        if area_obj is None:
            return MatchResult.no()
        if not any(
            af.lower() in area_obj.title.lower() or effective_area.startswith(af)
            for af in area_filters
        ):
            return MatchResult.no()

    # View filters (--inbox / --today / --someday / --evening)
    if getattr(args, "inbox", False) and not task.is_inbox:
        return MatchResult.no()
    if getattr(args, "today", False) and not task.is_today:
        return MatchResult.no()
    if getattr(args, "someday", False) and not task.in_someday:
        return MatchResult.no()
    if getattr(args, "evening", False) and not task.evening:
        return MatchResult.no()

    # --has-deadline / --no-deadline
    if getattr(args, "has_deadline", False) and task.deadline is None:
        return MatchResult.no()
    if getattr(args, "no_deadline", False) and task.deadline is not None:
        return MatchResult.no()

    # --recurring
    if getattr(args, "recurring", False) and not task.is_recurring:
        return MatchResult.no()

    # Date range filters — each flag may be specified multiple times (list of exprs)
    date_filters = [
        ("deadline", task.deadline),
        ("scheduled", task.start_date),
        ("created", task.creation_date),
        ("completed_on", task.stop_date),
    ]
    for flag_name, field_dt in date_filters:
        exprs: list[str] = getattr(args, flag_name, None) or []
        for expr in exprs:
            try:
                op, threshold = _parse_date_expr(
                    expr, f"--{flag_name.replace('_', '-')}"
                )
            except ValueError:
                # Already validated; treat as no-match to be safe
                return MatchResult.no()
            if not _date_matches(field_dt, op, threshold):
                return MatchResult.no()

    # --completed-on implies the task must be completed (checked before date filters)
    # This is already enforced above via _build_status_set, but guard here too in
    # case someone passes --completed-on alongside --any-status.
    completed_on_exprs_check = getattr(args, "completed_on", None) or []
    if (
        completed_on_exprs_check
        and not getattr(args, "any_status", False)
        and task.status != STATUS_COMPLETED
    ):
        return MatchResult.no()

    return MatchResult.yes(checklist_only=checklist_only)


# ---------------------------------------------------------------------------
# Output
# ---------------------------------------------------------------------------


def _fmt_result(
    task: Task,
    store: ThingsStore,
    id_prefix_len: int,
    detailed: bool,
) -> str:
    indent = "  "
    if task.is_project:
        return fmt_project_with_note(
            task,
            store,
            indent,
            id_prefix_len=id_prefix_len,
            show_indicators=True,
            detailed=detailed,
        )
    line = fmt_task_line(
        task,
        store,
        show_project=True,
        show_today_markers=True,
        id_prefix_len=id_prefix_len,
    )
    return fmt_task_with_note(
        line, task, indent, id_prefix_len=id_prefix_len, detailed=detailed
    )


# ---------------------------------------------------------------------------
# Command
# ---------------------------------------------------------------------------


def cmd_find(store: ThingsStore, args: argparse.Namespace) -> None:
    """Find tasks matching the given filters."""

    # Validate date expressions up front so errors appear before any output
    date_flag_names = ["deadline", "scheduled", "created", "completed_on"]
    for flag_name in date_flag_names:
        exprs: list[str] = getattr(args, flag_name, None) or []
        flag_label = f"--{flag_name.replace('_', '-')}"
        for expr in exprs:
            try:
                _parse_date_expr(expr, flag_label)
            except ValueError as e:
                print(str(e), file=sys.stderr)
                return

    # Resolve all tag filters up front; stash UUIDs on args for _matches
    tag_filters: list[str] = getattr(args, "tag", None) or []
    resolved_tag_uuids: list[str] = []
    for tag_filter in tag_filters:
        tag, err = _resolve_single_tag(store, tag_filter)
        if err:
            print(err, file=sys.stderr)
            return
        if tag is not None:
            resolved_tag_uuids.append(tag.uuid)
    args._resolved_tag_uuids = resolved_tag_uuids

    # Gather all candidates, retaining the match reason
    matched: list[tuple[Task, MatchResult]] = []
    for task in store._tasks.values():
        result = _matches(task, store, args)
        if result.matched:
            matched.append((task, result))

    # Sort: projects first (by index), then tasks (by index)
    matched.sort(key=lambda pair: (0 if pair[0].is_project else 1, pair[0].index))

    detailed = getattr(args, "detailed", False)
    count = len(matched)

    if not matched:
        print(colored("No matching tasks.", DIM))
        return

    id_prefix_len = store.unique_prefix_length([t.uuid for t, _ in matched])

    label = "task" if count == 1 else "tasks"
    print(colored(f"{ICONS.tag} Find  ({count} {label})", BOLD + CYAN))
    print()

    for task, match_result in matched:
        # Force detailed rendering when the match came from a checklist item
        force_detailed = detailed or match_result.checklist_only
        print(_fmt_result(task, store, id_prefix_len, force_detailed))


# ---------------------------------------------------------------------------
# Registration
# ---------------------------------------------------------------------------


def register(subparsers) -> dict[str, CommandHandler]:
    p = subparsers.add_parser(
        "find",
        help="Search and filter tasks",
        parents=[detailed_parent],
        formatter_class=argparse.RawDescriptionHelpFormatter,
        description="""Search and filter tasks.

Date filter syntax:  --deadline OP DATE
  OP is one of: >  <  >=  <=  =
  DATE is YYYY-MM-DD or a keyword: today, tomorrow, yesterday

  Examples:
    --deadline "<today"          overdue tasks
    --deadline ">=2026-01-01"    deadline on or after date
    --created ">=2026-01-01" --created "<=2026-03-31"   date range
""",
    )

    # Positional query
    p.add_argument(
        "query",
        nargs="?",
        default=None,
        help="Case-insensitive substring to match against task title",
    )

    # Status
    status_group = p.add_mutually_exclusive_group()
    status_group.add_argument(
        "--incomplete",
        action="store_true",
        default=False,
        help="Only incomplete tasks (default)",
    )
    status_group.add_argument(
        "--completed",
        action="store_true",
        default=False,
        help="Only completed tasks",
    )
    status_group.add_argument(
        "--canceled",
        action="store_true",
        default=False,
        help="Only canceled tasks",
    )
    status_group.add_argument(
        "--any-status",
        dest="any_status",
        action="store_true",
        default=False,
        help="Match tasks regardless of status",
    )

    # Location
    p.add_argument(
        "--tag",
        action="append",
        metavar="TAG",
        default=None,
        help="Has this tag (title or UUID prefix); repeatable, OR logic",
    )
    p.add_argument(
        "--project",
        action="append",
        metavar="PROJECT",
        default=None,
        help="In this project (title substring or UUID prefix); repeatable, OR logic",
    )
    p.add_argument(
        "--area",
        action="append",
        metavar="AREA",
        default=None,
        help="In this area (title substring or UUID prefix); repeatable, OR logic",
    )
    p.add_argument(
        "--inbox",
        action="store_true",
        default=False,
        help="In Inbox view",
    )
    p.add_argument(
        "--today",
        action="store_true",
        default=False,
        help="In Today view",
    )
    p.add_argument(
        "--someday",
        action="store_true",
        default=False,
        help="In Someday",
    )
    p.add_argument(
        "--evening",
        action="store_true",
        default=False,
        help="Evening flag set",
    )

    # Deadline presence
    deadline_group = p.add_mutually_exclusive_group()
    deadline_group.add_argument(
        "--has-deadline",
        dest="has_deadline",
        action="store_true",
        default=False,
        help="Has any deadline set",
    )
    deadline_group.add_argument(
        "--no-deadline",
        dest="no_deadline",
        action="store_true",
        default=False,
        help="No deadline set",
    )

    # Date filters (append so flag may be given multiple times)
    p.add_argument(
        "--deadline",
        action="append",
        metavar="EXPR",
        default=None,
        help="Deadline filter, e.g. '<today' or '>=2026-04-01' (repeatable for range)",
    )
    p.add_argument(
        "--scheduled",
        action="append",
        metavar="EXPR",
        default=None,
        help="Scheduled start date filter (repeatable)",
    )
    p.add_argument(
        "--created",
        action="append",
        metavar="EXPR",
        default=None,
        help="Creation date filter (repeatable)",
    )
    p.add_argument(
        "--completed-on",
        dest="completed_on",
        action="append",
        metavar="EXPR",
        default=None,
        help="Completion date filter; implies --completed (repeatable)",
    )

    # Other
    p.add_argument(
        "--notes",
        action="store_true",
        default=False,
        help="Also search query against note text",
    )
    p.add_argument(
        "--checklists",
        action="store_true",
        default=False,
        help="Also search query against checklist item titles; implies --detailed for checklist-only matches",
    )
    p.add_argument(
        "--recurring",
        action="store_true",
        default=False,
        help="Only recurring tasks",
    )

    return {"find": _adapt_store_command(cmd_find)}
