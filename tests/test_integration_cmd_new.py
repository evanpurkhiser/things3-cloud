from __future__ import annotations

from dataclasses import asdict
from datetime import datetime, timezone

from things_cloud.cli.common import _day_to_timestamp, _task6_note
from things_cloud.schema import TaskProps
from tests.mutating_fixtures import area, project, store, tag, task, today_ts
from tests.mutating_http_helpers import (
    assert_commit_payloads,
    assert_no_commits,
    p,
    run_cli_mutating_http,
)


NOW = 1_700_000_000.0
NEW_UUID = "MpkEei6ybkFS2n6SXvwfLf"
INBOX_ANCHOR_UUID = "A7h5eCi24RvAWKC3Hv3muf"
INBOX_OTHER_UUID = "KGvAPpMrzHAKMdgMiERP1V"
PROJECT_UUID = "JFdhhhp37fpryAKu8UXwzK"
AREA_UUID = "74rgJf6Qh9wYp2TcVk8mNB"
TAG_A_UUID = "By8mN2qRk5Wv7Xc9Dt3HpL"
TAG_B_UUID = "Cv9nP3sTk6Xw8Yd4Eu5JqM"


def _base_new_props(title: str) -> dict:
    props = asdict(TaskProps())
    props.update(
        {
            "tt": title,
            "tp": 0,
            "ss": 0,
            "st": 0,
            "tr": False,
            "cd": NOW,
            "md": NOW,
            "nt": None,
            "xx": {"_t": "oo", "sn": {}},
            "rmd": None,
            "rp": None,
        }
    )
    return props


def test_bare_create_payload() -> None:
    test_store = store()
    result = run_cli_mutating_http(
        'new "Ship release"',
        test_store,
        extra_patches=[
            p("things_cloud.cli.cmd_new.random_task_id", return_value=NEW_UUID),
            p("things_cloud.cli.cmd_new.time.time", return_value=NOW),
        ],
    )
    expected_props = _base_new_props("Ship release")
    assert result.stderr == ""
    assert_commit_payloads(
        result,
        {NEW_UUID: {"t": 0, "e": "Task6", "p": expected_props}},
    )


def test_when_variants_payloads() -> None:
    test_store = store()
    date_ts = _day_to_timestamp(datetime(2031, 4, 3, tzinfo=timezone.utc))
    cases = [
        ("today", {"st": 1, "sr": today_ts(), "tir": today_ts()}),
        ("someday", {"st": 2, "sr": None}),
        ("anytime", {"st": 1, "sr": None}),
        ("2031-04-03", {"st": 2, "sr": date_ts, "tir": date_ts}),
    ]
    for when, overrides in cases:
        result = run_cli_mutating_http(
            f'new "Task {when}" --when {when}',
            test_store,
            extra_patches=[
                p("things_cloud.cli.cmd_new.random_task_id", return_value=NEW_UUID),
                p("things_cloud.cli.cmd_new.time.time", return_value=NOW),
            ],
        )
        expected_props = _base_new_props(f"Task {when}")
        expected_props.update(overrides)
        assert result.stderr == ""
        assert_commit_payloads(
            result,
            {NEW_UUID: {"t": 0, "e": "Task6", "p": expected_props}},
        )


def test_notes_container_tags_deadline_payload() -> None:
    test_store = store(
        project(PROJECT_UUID, "Roadmap"),
        area(AREA_UUID, "Work"),
        tag(TAG_A_UUID, "urgent"),
        tag(TAG_B_UUID, "backend"),
    )
    deadline_ts = _day_to_timestamp(datetime(2032, 5, 6, tzinfo=timezone.utc))

    in_project = run_cli_mutating_http(
        f'new "Project task" --in {PROJECT_UUID} --notes "line one" --tags urgent,backend --deadline 2032-05-06',
        test_store,
        extra_patches=[
            p("things_cloud.cli.cmd_new.random_task_id", return_value=NEW_UUID),
            p("things_cloud.cli.cmd_new.time.time", return_value=NOW),
        ],
    )
    expected_project_props = _base_new_props("Project task")
    expected_project_props.update(
        {
            "nt": _task6_note("line one"),
            "pr": [PROJECT_UUID],
            "st": 1,
            "tg": [TAG_A_UUID, TAG_B_UUID],
            "dd": deadline_ts,
        }
    )
    assert_commit_payloads(
        in_project,
        {NEW_UUID: {"t": 0, "e": "Task6", "p": expected_project_props}},
    )

    in_area = run_cli_mutating_http(
        f'new "Area task" --in {AREA_UUID}',
        test_store,
        extra_patches=[
            p("things_cloud.cli.cmd_new.random_task_id", return_value=NEW_UUID),
            p("things_cloud.cli.cmd_new.time.time", return_value=NOW),
        ],
    )
    expected_area_props = _base_new_props("Area task")
    expected_area_props.update({"ar": [AREA_UUID], "st": 1})
    assert_commit_payloads(
        in_area,
        {NEW_UUID: {"t": 0, "e": "Task6", "p": expected_area_props}},
    )


def test_after_gap_payload() -> None:
    test_store = store(
        task(INBOX_ANCHOR_UUID, "Anchor", st=0, ix=1024),
        task(INBOX_OTHER_UUID, "Other", st=0, ix=2048),
    )
    result = run_cli_mutating_http(
        f'new "Inserted" --after {INBOX_ANCHOR_UUID}',
        test_store,
        extra_patches=[
            p("things_cloud.cli.cmd_new.random_task_id", return_value=NEW_UUID),
            p("things_cloud.cli.cmd_new.time.time", return_value=NOW),
        ],
    )

    expected_props = _base_new_props("Inserted")
    expected_props["ix"] = 1536
    assert_commit_payloads(
        result,
        {NEW_UUID: {"t": 0, "e": "Task6", "p": expected_props}},
    )


def test_after_rebalance_payload() -> None:
    test_store = store(
        task(INBOX_ANCHOR_UUID, "Anchor", st=0, ix=1024),
        task(INBOX_OTHER_UUID, "Other", st=0, ix=1025),
    )
    result = run_cli_mutating_http(
        f'new "Inserted" --after {INBOX_ANCHOR_UUID}',
        test_store,
        extra_patches=[
            p("things_cloud.cli.cmd_new.random_task_id", return_value=NEW_UUID),
            p("things_cloud.cli.cmd_new.time.time", return_value=NOW),
        ],
    )

    expected_props = _base_new_props("Inserted")
    expected_props["ix"] = 2048
    assert_commit_payloads(
        result,
        {
            NEW_UUID: {"t": 0, "e": "Task6", "p": expected_props},
            INBOX_OTHER_UUID: {"t": 1, "e": "Task6", "p": {"ix": 3072, "md": NOW}},
        },
    )


def test_when_today_after_today_anchor_payload() -> None:
    today = today_ts()
    test_store = store(
        task(
            INBOX_ANCHOR_UUID, "Today anchor", st=1, sr=today, tir=today, ti=25, ix=900
        ),
    )
    result = run_cli_mutating_http(
        f'new "Today inserted" --when today --after {INBOX_ANCHOR_UUID}',
        test_store,
        extra_patches=[
            p("things_cloud.cli.cmd_new.random_task_id", return_value=NEW_UUID),
            p("things_cloud.cli.cmd_new.time.time", return_value=NOW),
        ],
    )

    expected_props = _base_new_props("Today inserted")
    expected_props.update(
        {"st": 1, "sr": today, "tir": today, "ix": 901, "ti": 26, "sb": 0}
    )
    assert_commit_payloads(
        result,
        {NEW_UUID: {"t": 0, "e": "Task6", "p": expected_props}},
    )


def test_empty_title_is_rejected() -> None:
    result = run_cli_mutating_http('new "   "', store())
    assert_no_commits(result)
    assert result.stderr == "Task title cannot be empty.\n"


def test_unknown_container_is_rejected() -> None:
    result = run_cli_mutating_http('new "Ship" --in nope', store())
    assert_no_commits(result)
    assert result.stderr == "Container not found: nope\n"
