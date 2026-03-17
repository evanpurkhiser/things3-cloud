from __future__ import annotations

from tests.mutating_fixtures import store, tag
from tests.mutating_http_helpers import (
    assert_commit_payloads,
    assert_no_commits,
    p,
    run_cli_mutating_http,
)


NOW = 1_700_000_666.0
NEW_UUID = "A7h5eCi24RvAWKC3Hv3muf"


def test_new_area_payload() -> None:
    test_store = store()
    result = run_cli_mutating_http(
        'areas new "Personal"',
        test_store,
        extra_patches=[
            p("things_cloud.cli.cmd_areas.random_task_id", return_value=NEW_UUID),
            p("things_cloud.cli.cmd_areas.time.time", return_value=NOW),
        ],
    )
    assert_commit_payloads(
        result,
        {
            NEW_UUID: {
                "t": 0,
                "e": "Area3",
                "p": {
                    "tt": "Personal",
                    "ix": 0,
                    "xx": {"_t": "oo", "sn": {}},
                    "cd": NOW,
                    "md": NOW,
                },
            }
        },
    )


def test_new_area_with_tags_payload() -> None:
    TAG_UUID = "JWsQXoB8VgrfRgYFBmz2x8"
    test_store = store(tag(TAG_UUID, "focus"))
    result = run_cli_mutating_http(
        'areas new "Personal" --tags focus',
        test_store,
        extra_patches=[
            p("things_cloud.cli.cmd_areas.random_task_id", return_value=NEW_UUID),
            p("things_cloud.cli.cmd_areas.time.time", return_value=NOW),
        ],
    )
    assert_commit_payloads(
        result,
        {
            NEW_UUID: {
                "t": 0,
                "e": "Area3",
                "p": {
                    "tt": "Personal",
                    "ix": 0,
                    "xx": {"_t": "oo", "sn": {}},
                    "cd": NOW,
                    "md": NOW,
                    "tg": [TAG_UUID],
                },
            }
        },
    )


def test_empty_area_title_is_rejected() -> None:
    result = run_cli_mutating_http('areas new "   "', store())
    assert_no_commits(result)
    assert result.stderr == "Area title cannot be empty.\n"
