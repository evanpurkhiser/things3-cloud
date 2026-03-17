from __future__ import annotations

from datetime import datetime, timezone

from things_cloud.cli.common import _day_to_timestamp, _task6_note
from tests.helpers import build_store_from_journal
from tests.mutating_http_helpers import p, run_cli_mutating_http


NOW = 1_700_000_555.0
NEW_UUID = "A7h5eCi24RvAWKC3Hv3muf"
AREA_UUID = "KGvAPpMrzHAKMdgMiERP1V"
TAG_A_UUID = "MpkEei6ybkFS2n6SXvwfLf"
TAG_B_UUID = "JFdhhhp37fpryAKu8UXwzK"


def _area(uuid: str, title: str, **props) -> dict:
    base = {"tt": title, "ix": 0}
    base.update(props)
    return {uuid: {"t": 0, "e": "Area3", "p": base}}


def _tag(uuid: str, title: str, **props) -> dict:
    base = {"tt": title, "ix": 0}
    base.update(props)
    return {uuid: {"t": 0, "e": "Tag4", "p": base}}


def test_integration_cmd_projects_new_payload() -> None:
    store = build_store_from_journal(
        [
            _area(AREA_UUID, "Work"),
            _tag(TAG_A_UUID, "urgent"),
            _tag(TAG_B_UUID, "backend"),
        ]
    )
    deadline_ts = _day_to_timestamp(datetime(2035, 7, 8, tzinfo=timezone.utc))
    today = int(
        datetime.now(tz=timezone.utc)
        .replace(hour=0, minute=0, second=0, microsecond=0)
        .timestamp()
    )

    result = run_cli_mutating_http(
        f'projects new "Launch v2" --area {AREA_UUID} --when today --tags urgent,backend --deadline 2035-07-08 --notes "ship list"',
        store,
        extra_patches=[
            p("things_cloud.cli.cmd_projects.random_task_id", return_value=NEW_UUID),
            p("things_cloud.cli.cmd_projects.time.time", return_value=NOW),
        ],
    )

    assert result.commits[0].payload == {
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
    }
