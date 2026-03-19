"""Integration tests for the `find` command."""

from __future__ import annotations

from datetime import datetime, timezone

import pytest

from tests.helpers import build_store_from_journal, run_cli


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _ts(year: int, month: int, day: int) -> int:
    """Return a UTC-midnight Unix timestamp for the given date."""
    return int(datetime(year, month, day, 0, 0, 0, tzinfo=timezone.utc).timestamp())


def _task(
    uuid: str,
    title: str,
    *,
    st: int = 1,  # 0=Inbox, 1=Anytime, 2=Someday
    ss: int = 0,  # 0=incomplete, 2=canceled, 3=completed
    ix: int = 10,
    tp: int = 0,  # 0=todo, 1=project
    sr: int | None = None,
    dd: int | None = None,
    sp: int | None = None,
    cd: int | None = None,
    notes: str | None = None,
    tags: list[str] | None = None,
    area: str | None = None,
    project: str | None = None,
    sb: int = 0,  # evening flag
    rr: dict | None = None,
) -> dict:
    props: dict = {
        "tt": title,
        "st": st,
        "ss": ss,
        "tp": tp,
        "ix": ix,
        "cd": cd if cd is not None else _ts(2025, 1, 1),
        "md": _ts(2025, 1, 1),
    }
    if sr is not None:
        props["sr"] = sr
    if dd is not None:
        props["dd"] = dd
    if sp is not None:
        props["sp"] = sp
    if notes is not None:
        props["nt"] = {"_t": "tx", "t": 1, "ch": 0, "v": notes}
    if tags:
        props["tg"] = tags
    if area:
        props["ar"] = [area]
    if project:
        props["pr"] = [project]
    if sb:
        props["sb"] = sb
    if rr is not None:
        props["rr"] = rr
    return {uuid: {"t": 0, "e": "Task6", "p": props}}


def _area(uuid: str, title: str, *, ix: int = 10) -> dict:
    return {
        uuid: {
            "t": 0,
            "e": "Area3",
            "p": {"tt": title, "ix": ix},
        }
    }


def _tag(uuid: str, title: str, *, ix: int = 10) -> dict:
    return {
        uuid: {
            "t": 0,
            "e": "Tag4",
            "p": {"tt": title, "ix": ix},
        }
    }


def _checklist_item(
    uuid: str, task_uuid: str, title: str, *, ix: int, ss: int = 0
) -> dict:
    return {
        uuid: {
            "t": 0,
            "e": "ChecklistItem3",
            "p": {
                "tt": title,
                "ts": [task_uuid],
                "ss": ss,
                "ix": ix,
                "cd": _ts(2025, 1, 1),
                "md": _ts(2025, 1, 1),
            },
        }
    }


def store(journal: list[dict]):
    return build_store_from_journal(journal)


# ---------------------------------------------------------------------------
# Empty results
# ---------------------------------------------------------------------------


def test_find_no_results_empty_store() -> None:
    out = run_cli("find", store([]))
    assert "No matching tasks." in out


def test_find_query_no_match() -> None:
    journal = [_task("AAA111111111111111111111", "Buy groceries")]
    out = run_cli("find zzznomatch", store(journal))
    assert "No matching tasks." in out


# ---------------------------------------------------------------------------
# Title query
# ---------------------------------------------------------------------------


def test_find_title_query_case_insensitive(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "Buy Groceries", ix=10),
        _task("BBB111111111111111111111", "Call doctor", ix=20),
        _task("CCC111111111111111111111", "grocery run", ix=30),
    ]
    # "grocer" is a substring of both "Groceries" and "grocery" (case-insensitive)
    out = run_cli("find grocer", store_from_journal(journal))
    assert "Buy Groceries" in out
    assert "grocery run" in out
    assert "Call doctor" not in out


def test_find_title_query_partial_match(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "Ship release v1.0", ix=10),
        _task("BBB111111111111111111111", "Release notes", ix=20),
        _task("CCC111111111111111111111", "Read book", ix=30),
    ]
    out = run_cli("find release", store_from_journal(journal))
    assert "Ship release v1.0" in out
    assert "Release notes" in out
    assert "Read book" not in out


# ---------------------------------------------------------------------------
# Notes search
# ---------------------------------------------------------------------------


def test_find_notes_not_searched_by_default(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "Task A", notes="contains dentist info"),
        _task("BBB111111111111111111111", "dentist appointment"),
    ]
    out = run_cli("find dentist", store_from_journal(journal))
    assert "Task A" not in out
    assert "dentist appointment" in out


def test_find_notes_searched_with_flag(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "Task A", notes="contains dentist info"),
        _task("BBB111111111111111111111", "dentist appointment"),
    ]
    out = run_cli("find dentist --notes", store_from_journal(journal))
    assert "Task A" in out
    assert "dentist appointment" in out


# ---------------------------------------------------------------------------
# Status filters
# ---------------------------------------------------------------------------


def test_find_default_shows_only_incomplete(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "Open task", ss=0),
        _task("BBB111111111111111111111", "Done task", ss=3, sp=_ts(2025, 3, 1)),
        _task("CCC111111111111111111111", "Canceled task", ss=2, sp=_ts(2025, 3, 1)),
    ]
    out = run_cli("find", store_from_journal(journal))
    assert "Open task" in out
    assert "Done task" not in out
    assert "Canceled task" not in out


def test_find_completed_flag(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "Open task", ss=0),
        _task("BBB111111111111111111111", "Done task", ss=3, sp=_ts(2025, 3, 1)),
        _task("CCC111111111111111111111", "Canceled task", ss=2, sp=_ts(2025, 3, 1)),
    ]
    out = run_cli("find --completed", store_from_journal(journal))
    assert "Open task" not in out
    assert "Done task" in out
    assert "Canceled task" not in out


def test_find_canceled_flag(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "Open task", ss=0),
        _task("BBB111111111111111111111", "Done task", ss=3, sp=_ts(2025, 3, 1)),
        _task("CCC111111111111111111111", "Canceled task", ss=2, sp=_ts(2025, 3, 1)),
    ]
    out = run_cli("find --canceled", store_from_journal(journal))
    assert "Open task" not in out
    assert "Done task" not in out
    assert "Canceled task" in out


def test_find_any_status(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "Open task", ss=0),
        _task("BBB111111111111111111111", "Done task", ss=3, sp=_ts(2025, 3, 1)),
        _task("CCC111111111111111111111", "Canceled task", ss=2, sp=_ts(2025, 3, 1)),
    ]
    out = run_cli("find --any-status", store_from_journal(journal))
    assert "Open task" in out
    assert "Done task" in out
    assert "Canceled task" in out


# ---------------------------------------------------------------------------
# Tag filter
# ---------------------------------------------------------------------------


def test_find_tag_filter_by_title(store_from_journal) -> None:
    journal = [
        _tag("TTAG1111111111111111111T", "Work"),
        _task(
            "AAA111111111111111111111", "Work report", tags=["TTAG1111111111111111111T"]
        ),
        _task("BBB111111111111111111111", "Personal errand"),
    ]
    out = run_cli("find --tag Work", store_from_journal(journal))
    assert "Work report" in out
    assert "Personal errand" not in out


def test_find_tag_not_found_error(store_from_journal) -> None:
    """Tag not found: error goes to stderr, no tasks appear in stdout."""
    journal = [_task("AAA111111111111111111111", "Some task")]
    out = run_cli("find --tag NoSuchTag", store_from_journal(journal))
    # Error is emitted to stderr; stdout should be empty (early return, no tasks printed)
    assert "Some task" not in out


def test_find_tag_case_insensitive(store_from_journal) -> None:
    journal = [
        _tag("TTAG1111111111111111111T", "Work"),
        _task(
            "AAA111111111111111111111", "Work report", tags=["TTAG1111111111111111111T"]
        ),
        _task("BBB111111111111111111111", "Personal errand"),
    ]
    out = run_cli("find --tag work", store_from_journal(journal))
    assert "Work report" in out
    assert "Personal errand" not in out


def test_find_tag_multiple_or(store_from_journal) -> None:
    """Multiple --tag values are OR'd: task matching any tag is included."""
    journal = [
        _tag("TTAG1111111111111111111T", "Work"),
        _tag("TTAG2222222222222222222T", "Urgent"),
        _task(
            "AAA111111111111111111111", "Work task", tags=["TTAG1111111111111111111T"]
        ),
        _task(
            "BBB111111111111111111111", "Urgent task", tags=["TTAG2222222222222222222T"]
        ),
        _task("CCC111111111111111111111", "Untagged task"),
    ]
    out = run_cli("find --tag Work --tag Urgent", store_from_journal(journal))
    assert "Work task" in out
    assert "Urgent task" in out
    assert "Untagged task" not in out


# ---------------------------------------------------------------------------
# Project filter
# ---------------------------------------------------------------------------


def test_find_project_filter_by_title_substring(store_from_journal) -> None:
    proj_uuid = "PPROJECT11111111111111111"
    journal = [
        _task(proj_uuid, "Q1 Launch", tp=1, st=1),
        _task("AAA111111111111111111111", "Write spec", project=proj_uuid),
        _task("BBB111111111111111111111", "Send email"),
    ]
    out = run_cli("find --project Launch", store_from_journal(journal))
    assert "Write spec" in out
    assert "Send email" not in out


def test_find_project_filter_no_match(store_from_journal) -> None:
    proj_uuid = "PPROJECT11111111111111111"
    journal = [
        _task(proj_uuid, "Q1 Launch", tp=1, st=1),
        _task("AAA111111111111111111111", "Write spec", project=proj_uuid),
    ]
    out = run_cli("find --project NoSuchProject", store_from_journal(journal))
    assert "No matching tasks." in out


def test_find_project_multiple_or(store_from_journal) -> None:
    """Multiple --project values are OR'd: task in any matching project is included."""
    proj1 = "PPROJECT11111111111111111"
    proj2 = "PPROJECT22222222222222222"
    journal = [
        _task(proj1, "Alpha Project", tp=1, st=1),
        _task(proj2, "Beta Project", tp=1, st=1),
        _task("AAA111111111111111111111", "Alpha task", project=proj1),
        _task("BBB111111111111111111111", "Beta task", project=proj2),
        _task("CCC111111111111111111111", "Unscoped task"),
    ]
    out = run_cli("find --project Alpha --project Beta", store_from_journal(journal))
    assert "Alpha task" in out
    assert "Beta task" in out
    assert "Unscoped task" not in out


# ---------------------------------------------------------------------------
# Area filter
# ---------------------------------------------------------------------------


def test_find_area_filter_by_title_substring(store_from_journal) -> None:
    area_uuid = "AAREA111111111111111111A"
    journal = [
        _area(area_uuid, "Personal"),
        _task("AAA111111111111111111111", "Read book", area=area_uuid),
        _task("BBB111111111111111111111", "Write report"),
    ]
    out = run_cli("find --area Personal", store_from_journal(journal))
    assert "Read book" in out
    assert "Write report" not in out


def test_find_area_filter_no_match(store_from_journal) -> None:
    area_uuid = "AAREA111111111111111111A"
    journal = [
        _area(area_uuid, "Personal"),
        _task("AAA111111111111111111111", "Read book", area=area_uuid),
    ]
    out = run_cli("find --area Work", store_from_journal(journal))
    assert "No matching tasks." in out


def test_find_area_filter_partial_match(store_from_journal) -> None:
    area_uuid = "AAREA111111111111111111A"
    journal = [
        _area(area_uuid, "Work Projects"),
        _task("AAA111111111111111111111", "Write spec", area=area_uuid),
        _task("BBB111111111111111111111", "Buy groceries"),
    ]
    out = run_cli("find --area Work", store_from_journal(journal))
    assert "Write spec" in out
    assert "Buy groceries" not in out


def test_find_area_multiple_or(store_from_journal) -> None:
    """Multiple --area values are OR'd: task in any matching area is included."""
    area1 = "AAREA111111111111111111A"
    area2 = "AAREA222222222222222222A"
    journal = [
        _area(area1, "Work"),
        _area(area2, "Personal"),
        _task("AAA111111111111111111111", "Work task", area=area1),
        _task("BBB111111111111111111111", "Personal task", area=area2),
        _task("CCC111111111111111111111", "Unscoped task"),
    ]
    out = run_cli("find --area Work --area Personal", store_from_journal(journal))
    assert "Work task" in out
    assert "Personal task" in out
    assert "Unscoped task" not in out


# ---------------------------------------------------------------------------
# View flags: --inbox / --today / --someday / --evening
# ---------------------------------------------------------------------------


def test_find_inbox_flag(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "Inbox task", st=0),
        _task("BBB111111111111111111111", "Anytime task", st=1),
    ]
    out = run_cli("find --inbox", store_from_journal(journal))
    assert "Inbox task" in out
    assert "Anytime task" not in out


def test_find_someday_flag(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "Someday dream", st=2),
        _task("BBB111111111111111111111", "Anytime task", st=1),
    ]
    out = run_cli("find --someday", store_from_journal(journal))
    assert "Someday dream" in out
    assert "Anytime task" not in out


def test_find_evening_flag(store_from_journal) -> None:
    today_ts = _ts(2025, 3, 18)
    journal = [
        _task("AAA111111111111111111111", "Evening walk", st=1, sr=today_ts, sb=1),
        _task("BBB111111111111111111111", "Morning run", st=1, sr=today_ts, sb=0),
    ]
    out = run_cli("find --evening", store_from_journal(journal))
    assert "Evening walk" in out
    assert "Morning run" not in out


# ---------------------------------------------------------------------------
# --has-deadline / --no-deadline
# ---------------------------------------------------------------------------


def test_find_has_deadline(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "With deadline", dd=_ts(2026, 6, 1)),
        _task("BBB111111111111111111111", "No deadline"),
    ]
    out = run_cli("find --has-deadline", store_from_journal(journal))
    assert "With deadline" in out
    assert "No deadline" not in out


def test_find_no_deadline(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "With deadline", dd=_ts(2026, 6, 1)),
        _task("BBB111111111111111111111", "No deadline"),
    ]
    out = run_cli("find --no-deadline", store_from_journal(journal))
    assert "No deadline" in out
    assert "With deadline" not in out


# ---------------------------------------------------------------------------
# --deadline date filter
# ---------------------------------------------------------------------------


def test_find_deadline_lt(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "Past due", dd=_ts(2025, 1, 1)),
        _task("BBB111111111111111111111", "Future", dd=_ts(2027, 1, 1)),
        _task("CCC111111111111111111111", "No deadline"),
    ]
    out = run_cli("find --deadline '<2026-01-01'", store_from_journal(journal))
    assert "Past due" in out
    assert "Future" not in out
    assert "No deadline" not in out


def test_find_deadline_lte(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "On the date", dd=_ts(2026, 4, 1)),
        _task("BBB111111111111111111111", "Before", dd=_ts(2026, 3, 31)),
        _task("CCC111111111111111111111", "After", dd=_ts(2026, 4, 2)),
    ]
    out = run_cli("find --deadline '<=2026-04-01'", store_from_journal(journal))
    assert "On the date" in out
    assert "Before" in out
    assert "After" not in out


def test_find_deadline_gt(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "Future", dd=_ts(2027, 1, 1)),
        _task("BBB111111111111111111111", "Past", dd=_ts(2024, 1, 1)),
    ]
    out = run_cli("find --deadline '>2026-01-01'", store_from_journal(journal))
    assert "Future" in out
    assert "Past" not in out


def test_find_deadline_gte(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "On the date", dd=_ts(2026, 4, 1)),
        _task("BBB111111111111111111111", "After", dd=_ts(2026, 4, 2)),
        _task("CCC111111111111111111111", "Before", dd=_ts(2026, 3, 31)),
    ]
    out = run_cli("find --deadline '>=2026-04-01'", store_from_journal(journal))
    assert "On the date" in out
    assert "After" in out
    assert "Before" not in out


def test_find_deadline_eq(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "Exact match", dd=_ts(2026, 4, 1)),
        _task("BBB111111111111111111111", "Wrong date", dd=_ts(2026, 4, 2)),
    ]
    out = run_cli("find --deadline '=2026-04-01'", store_from_journal(journal))
    assert "Exact match" in out
    assert "Wrong date" not in out


def test_find_deadline_range(store_from_journal) -> None:
    """Two --deadline flags form an AND range."""
    journal = [
        _task("AAA111111111111111111111", "In range", dd=_ts(2026, 3, 15)),
        _task("BBB111111111111111111111", "Too early", dd=_ts(2026, 2, 28)),
        _task("CCC111111111111111111111", "Too late", dd=_ts(2026, 4, 2)),
    ]
    out = run_cli(
        "find --deadline '>=2026-03-01' --deadline '<=2026-03-31'",
        store_from_journal(journal),
    )
    assert "In range" in out
    assert "Too early" not in out
    assert "Too late" not in out


# ---------------------------------------------------------------------------
# --scheduled date filter
# ---------------------------------------------------------------------------


def test_find_scheduled_gte(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "Future scheduled", sr=_ts(2026, 6, 1)),
        _task("BBB111111111111111111111", "Past scheduled", sr=_ts(2025, 1, 1)),
        _task("CCC111111111111111111111", "No schedule"),
    ]
    out = run_cli("find --scheduled '>=2026-01-01'", store_from_journal(journal))
    assert "Future scheduled" in out
    assert "Past scheduled" not in out
    assert "No schedule" not in out


def test_find_scheduled_lt(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "Old", sr=_ts(2024, 6, 1)),
        _task("BBB111111111111111111111", "New", sr=_ts(2026, 6, 1)),
    ]
    out = run_cli("find --scheduled '<2025-01-01'", store_from_journal(journal))
    assert "Old" in out
    assert "New" not in out


# ---------------------------------------------------------------------------
# --created date filter
# ---------------------------------------------------------------------------


def test_find_created_range(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "Jan task", cd=_ts(2026, 1, 15)),
        _task("BBB111111111111111111111", "Feb task", cd=_ts(2026, 2, 15)),
        _task("CCC111111111111111111111", "Mar task", cd=_ts(2026, 3, 15)),
    ]
    out = run_cli(
        "find --created '>=2026-02-01' --created '<=2026-02-28'",
        store_from_journal(journal),
    )
    assert "Jan task" not in out
    assert "Feb task" in out
    assert "Mar task" not in out


# ---------------------------------------------------------------------------
# --completed-on date filter
# ---------------------------------------------------------------------------


def test_find_completed_on_implies_completed(store_from_journal) -> None:
    """--completed-on should only return completed tasks, not incomplete ones."""
    journal = [
        _task("AAA111111111111111111111", "Done task", ss=3, sp=_ts(2026, 3, 1)),
        _task("BBB111111111111111111111", "Open task", ss=0),
    ]
    out = run_cli("find --completed-on '=2026-03-01'", store_from_journal(journal))
    assert "Done task" in out
    assert "Open task" not in out


def test_find_completed_on_range(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "Early done", ss=3, sp=_ts(2026, 2, 1)),
        _task("BBB111111111111111111111", "Mid done", ss=3, sp=_ts(2026, 3, 15)),
        _task("CCC111111111111111111111", "Late done", ss=3, sp=_ts(2026, 4, 1)),
    ]
    out = run_cli(
        "find --completed-on '>=2026-03-01' --completed-on '<=2026-03-31'",
        store_from_journal(journal),
    )
    assert "Early done" not in out
    assert "Mid done" in out
    assert "Late done" not in out


# ---------------------------------------------------------------------------
# --checklists flag
# ---------------------------------------------------------------------------


def test_find_checklists_not_searched_by_default(store_from_journal) -> None:
    task_uuid = "AAA111111111111111111111"
    journal = [
        _task(task_uuid, "Parent task"),
        _checklist_item(
            "CCC111111111111111111111", task_uuid, "dentist appointment", ix=1
        ),
    ]
    out = run_cli("find dentist", store_from_journal(journal))
    assert "Parent task" not in out


def test_find_checklists_searched_with_flag(store_from_journal) -> None:
    task_uuid = "AAA111111111111111111111"
    journal = [
        _task(task_uuid, "Parent task"),
        _checklist_item(
            "CCC111111111111111111111", task_uuid, "dentist appointment", ix=1
        ),
    ]
    out = run_cli("find dentist --checklists", store_from_journal(journal))
    assert "Parent task" in out


def test_find_checklists_match_implies_detailed(store_from_journal) -> None:
    """When a task matches only via a checklist item, it renders in detailed mode."""
    task_uuid = "AAA111111111111111111111"
    journal = [
        _task(task_uuid, "Parent task"),
        _checklist_item(
            "CCC111111111111111111111", task_uuid, "dentist appointment", ix=1
        ),
    ]
    out = run_cli("find dentist --checklists", store_from_journal(journal))
    # Detailed rendering shows the checklist item title
    assert "dentist appointment" in out


def test_find_checklists_title_match_does_not_force_detailed(
    store_from_journal,
) -> None:
    """When a task matches via title (not checklist), --detailed is not forced."""
    task_uuid = "AAA111111111111111111111"
    journal = [
        _task(task_uuid, "dentist task"),
        _checklist_item("CCC111111111111111111111", task_uuid, "call to confirm", ix=1),
    ]
    out = run_cli("find dentist --checklists", store_from_journal(journal))
    # Task appears but checklist item is NOT shown (no --detailed flag, title matched)
    assert "dentist task" in out
    assert "call to confirm" not in out


def test_find_checklists_title_match_with_detailed_shows_checklist(
    store_from_journal,
) -> None:
    """Title match + --detailed still shows checklist items."""
    task_uuid = "AAA111111111111111111111"
    journal = [
        _task(task_uuid, "dentist task"),
        _checklist_item("CCC111111111111111111111", task_uuid, "call to confirm", ix=1),
    ]
    out = run_cli("find dentist --checklists --detailed", store_from_journal(journal))
    assert "dentist task" in out
    assert "call to confirm" in out


def test_find_checklists_case_insensitive(store_from_journal) -> None:
    task_uuid = "AAA111111111111111111111"
    journal = [
        _task(task_uuid, "Parent task"),
        _checklist_item("CCC111111111111111111111", task_uuid, "Book Dentist", ix=1),
    ]
    out = run_cli("find dentist --checklists", store_from_journal(journal))
    assert "Parent task" in out
    assert "Book Dentist" in out


def test_find_checklists_partial_match(store_from_journal) -> None:
    """Multiple checklist items — only the matching one causes the task to surface."""
    task_uuid = "AAA111111111111111111111"
    other_uuid = "BBB111111111111111111111"
    journal = [
        _task(task_uuid, "Task with dentist item"),
        _checklist_item("CCC111111111111111111111", task_uuid, "Call dentist", ix=1),
        _checklist_item("DDD111111111111111111111", task_uuid, "Buy groceries", ix=2),
        _task(other_uuid, "Unrelated task"),
        _checklist_item("EEE111111111111111111111", other_uuid, "No match here", ix=1),
    ]
    out = run_cli("find dentist --checklists", store_from_journal(journal))
    assert "Task with dentist item" in out
    assert "Unrelated task" not in out


# ---------------------------------------------------------------------------
# Projects included by default
# ---------------------------------------------------------------------------


def test_find_projects_included_by_default(store_from_journal) -> None:
    journal = [
        _task("PPROJECT11111111111111111", "Big Project", tp=1, st=1),
        _task("AAA111111111111111111111", "A task"),
    ]
    out = run_cli("find", store_from_journal(journal))
    assert "Big Project" in out
    assert "A task" in out


def test_find_query_matches_project_title(store_from_journal) -> None:
    journal = [
        _task("PPROJECT11111111111111111", "Launch Project", tp=1, st=1),
        _task("AAA111111111111111111111", "Launch task"),
        _task("BBB111111111111111111111", "Unrelated task"),
    ]
    out = run_cli("find launch", store_from_journal(journal))
    assert "Launch Project" in out
    assert "Launch task" in out
    assert "Unrelated task" not in out


# ---------------------------------------------------------------------------
# --recurring flag
# ---------------------------------------------------------------------------


def test_find_recurring_flag(store_from_journal) -> None:
    journal = [
        _task(
            "AAA111111111111111111111",
            "Weekly review",
            rr={"fr": 0, "fu": 2, "fi": 1},
        ),
        _task("BBB111111111111111111111", "One-off task"),
    ]
    out = run_cli("find --recurring", store_from_journal(journal))
    assert "Weekly review" in out
    assert "One-off task" not in out


# ---------------------------------------------------------------------------
# Combined filters
# ---------------------------------------------------------------------------


def test_find_query_plus_tag(store_from_journal) -> None:
    journal = [
        _tag("TTAG1111111111111111111T", "Work"),
        _task(
            "AAA111111111111111111111",
            "Write report",
            tags=["TTAG1111111111111111111T"],
        ),
        _task("BBB111111111111111111111", "Write poem"),  # no tag
        _task(
            "CCC111111111111111111111",
            "Review PR",
            tags=["TTAG1111111111111111111T"],
        ),  # tagged but no query match
    ]
    out = run_cli("find write --tag Work", store_from_journal(journal))
    assert "Write report" in out
    assert "Write poem" not in out
    assert "Review PR" not in out


def test_find_query_plus_deadline_range(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "File taxes", dd=_ts(2026, 4, 15)),
        _task("BBB111111111111111111111", "File report", dd=_ts(2026, 6, 1)),
        _task("CCC111111111111111111111", "Buy flowers"),
    ]
    out = run_cli(
        "find file --deadline '<=2026-05-01'",
        store_from_journal(journal),
    )
    assert "File taxes" in out
    assert "File report" not in out
    assert "Buy flowers" not in out


def test_find_area_plus_tag(store_from_journal) -> None:
    area_uuid = "AAREA111111111111111111A"
    journal = [
        _area(area_uuid, "Work"),
        _tag("TTAG1111111111111111111T", "Urgent"),
        _task(
            "AAA111111111111111111111",
            "Urgent work task",
            area=area_uuid,
            tags=["TTAG1111111111111111111T"],
        ),
        _task("BBB111111111111111111111", "Work task no tag", area=area_uuid),
        _task(
            "CCC111111111111111111111",
            "Non-work urgent",
            tags=["TTAG1111111111111111111T"],
        ),
    ]
    out = run_cli("find --area Work --tag Urgent", store_from_journal(journal))
    assert "Urgent work task" in out
    assert "Work task no tag" not in out
    assert "Non-work urgent" not in out


def test_find_status_plus_area(store_from_journal) -> None:
    area_uuid = "AAREA111111111111111111A"
    journal = [
        _area(area_uuid, "Health"),
        _task(
            "AAA111111111111111111111",
            "Done health task",
            ss=3,
            sp=_ts(2025, 3, 1),
            area=area_uuid,
        ),
        _task(
            "BBB111111111111111111111",
            "Open health task",
            ss=0,
            area=area_uuid,
        ),
        _task("CCC111111111111111111111", "Done elsewhere", ss=3, sp=_ts(2025, 3, 1)),
    ]
    out = run_cli("find --completed --area Health", store_from_journal(journal))
    assert "Done health task" in out
    assert "Open health task" not in out
    assert "Done elsewhere" not in out


# ---------------------------------------------------------------------------
# Output formatting
# ---------------------------------------------------------------------------


def test_find_shows_count_in_header(store_from_journal) -> None:
    journal = [
        _task("AAA111111111111111111111", "Task one"),
        _task("BBB111111111111111111111", "Task two"),
    ]
    out = run_cli("find", store_from_journal(journal))
    assert "2 tasks" in out


def test_find_singular_task_label(store_from_journal) -> None:
    journal = [_task("AAA111111111111111111111", "Only task")]
    out = run_cli("find", store_from_journal(journal))
    assert "1 task" in out
    assert "1 tasks" not in out


def test_find_detailed_shows_notes(store_from_journal) -> None:
    journal = [
        _task(
            "AAA111111111111111111111",
            "Task with note",
            notes="important detail",
        )
    ]
    out = run_cli("find --detailed", store_from_journal(journal))
    assert "Task with note" in out
    assert "important detail" in out


def test_find_shows_project_in_task_line(store_from_journal) -> None:
    proj_uuid = "PPROJECT11111111111111111"
    journal = [
        _task(proj_uuid, "My Project", tp=1, st=1),
        _task("AAA111111111111111111111", "Project task", project=proj_uuid),
    ]
    out = run_cli("find", store_from_journal(journal))
    assert "Project task" in out
    assert "My Project" in out


# ---------------------------------------------------------------------------
# Error handling
# ---------------------------------------------------------------------------


def test_find_invalid_date_expr(store_from_journal) -> None:
    """Bad date expression should not crash, just emit an error."""
    journal = [_task("AAA111111111111111111111", "A task")]
    # run_cli captures stdout; errors go to stderr which we can detect by absence of results
    out = run_cli("find --deadline 'bad-expr'", store_from_journal(journal))
    # Should not show any tasks (early return on parse error)
    assert "A task" not in out


def test_find_trashed_tasks_excluded(store_from_journal) -> None:
    journal = [
        {
            "AAA111111111111111111111": {
                "t": 0,
                "e": "Task6",
                "p": {
                    "tt": "Trashed task",
                    "st": 1,
                    "ss": 0,
                    "tp": 0,
                    "ix": 10,
                    "cd": _ts(2025, 1, 1),
                    "md": _ts(2025, 1, 1),
                    "tr": True,
                },
            }
        }
    ]
    out = run_cli("find", store_from_journal(journal))
    assert "Trashed task" not in out


def test_find_headings_excluded(store_from_journal) -> None:
    journal = [
        {
            "AAA111111111111111111111": {
                "t": 0,
                "e": "Task6",
                "p": {
                    "tt": "A Heading",
                    "st": 1,
                    "ss": 0,
                    "tp": 2,  # heading
                    "ix": 10,
                    "cd": _ts(2025, 1, 1),
                    "md": _ts(2025, 1, 1),
                },
            }
        }
    ]
    out = run_cli("find", store_from_journal(journal))
    assert "A Heading" not in out
