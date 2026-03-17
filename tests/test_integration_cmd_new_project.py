from __future__ import annotations

from datetime import datetime, timezone

from things_cloud.cli.common import _day_to_timestamp, _task6_note
from tests.mutating_fixtures import area, store, tag, today_ts
from tests.mutating_http_helpers import (
    assert_commit_payloads,
    assert_no_commits,
    p,
    run_cli_mutating_http,
)


NOW = 1_700_000_555.0
NEW_UUID = "A7h5eCi24RvAWKC3Hv3muf"
AREA_UUID = "KGvAPpMrzHAKMdgMiERP1V"
TAG_A_UUID = "MpkEei6ybkFS2n6SXvwfLf"
TAG_B_UUID = "JFdhhhp37fpryAKu8UXwzK"


def test_new_project_payload() -> None:
    test_store = store(
        area(AREA_UUID, "Work"),
        tag(TAG_A_UUID, "urgent"),
        tag(TAG_B_UUID, "backend"),
    )
    deadline_ts = _day_to_timestamp(datetime(2035, 7, 8, tzinfo=timezone.utc))
    today = today_ts()

    result = run_cli_mutating_http(
        f'projects new "Launch v2" --area {AREA_UUID} --when today --tags urgent,backend --deadline 2035-07-08 --notes "ship list"',
        test_store,
        extra_patches=[
            p("things_cloud.cli.cmd_projects.random_task_id", return_value=NEW_UUID),
            p("things_cloud.cli.cmd_projects.time.time", return_value=NOW),
        ],
    )

    assert_commit_payloads(
        result,
        {
            NEW_UUID: {
                "t": 0,
                "e": "Task6",
                "p": {
                    "tt": "Launch v2",
                    "tp": 1,
                    "ss": 0,
                    "st": 1,
                    "tr": False,
                    "cd": NOW,
                    "md": NOW,
                    "nt": _task6_note("ship list"),
                    "xx": {"_t": "oo", "sn": {}},
                    "icp": True,
                    "rmd": None,
                    "rp": None,
                    "ar": [AREA_UUID],
                    "sr": today,
                    "tir": today,
                    "tg": [TAG_A_UUID, TAG_B_UUID],
                    "dd": deadline_ts,
                },
            }
        },
    )


def test_empty_project_title_is_rejected() -> None:
    result = run_cli_mutating_http('projects new "   "', store())
    assert_no_commits(result)
    assert result.stderr == "Project title cannot be empty.\n"
