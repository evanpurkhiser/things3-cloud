from __future__ import annotations

from tests.helpers import build_store_from_journal
from tests.mutating_http_helpers import p, run_cli_mutating_http


NOW = 1_700_000_666.0
NEW_UUID = "A7h5eCi24RvAWKC3Hv3muf"


def test_new_area_payload() -> None:
    store = build_store_from_journal([])
    result = run_cli_mutating_http(
        'areas new "Personal"',
        store,
        extra_patches=[
            p("things_cloud.cli.cmd_areas.random_task_id", return_value=NEW_UUID),
            p("things_cloud.cli.cmd_areas.time.time", return_value=NOW),
        ],
    )
    assert result.commits[0].payload == {
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
    }
