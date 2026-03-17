from __future__ import annotations

from datetime import datetime, timezone

from things_cloud.store import ThingsStore
from tests.helpers import build_store_from_journal


def today_ts() -> int:
    return int(
        datetime.now(tz=timezone.utc)
        .replace(hour=0, minute=0, second=0, microsecond=0)
        .timestamp()
    )


def store(*entries: dict) -> ThingsStore:
    return build_store_from_journal(list(entries))


def task(uuid: str, title: str, **props) -> dict:
    base = {"tt": title, "tp": 0, "ss": 0, "st": 0, "ix": 0, "cd": 1, "md": 1}
    base.update(props)
    return {uuid: {"t": 0, "e": "Task6", "p": base}}


def project(uuid: str, title: str, **props) -> dict:
    base = {"tt": title, "tp": 1, "ss": 0, "st": 1, "ix": 0, "cd": 1, "md": 1}
    base.update(props)
    return {uuid: {"t": 0, "e": "Task6", "p": base}}


def area(uuid: str, title: str, **props) -> dict:
    base = {"tt": title, "ix": 0}
    base.update(props)
    return {uuid: {"t": 0, "e": "Area3", "p": base}}


def tag(uuid: str, title: str, **props) -> dict:
    base = {"tt": title, "ix": 0}
    base.update(props)
    return {uuid: {"t": 0, "e": "Tag4", "p": base}}


def checklist(uuid: str, task_uuid: str, title: str, **props) -> dict:
    base = {"tt": title, "ts": [task_uuid], "ss": 0, "ix": 0, "cd": 1, "md": 1}
    base.update(props)
    return {uuid: {"t": 0, "e": "ChecklistItem3", "p": base}}
