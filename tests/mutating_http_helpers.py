from __future__ import annotations

import io
import json
import re
import shlex
from contextlib import ExitStack, redirect_stderr, redirect_stdout
from dataclasses import dataclass
from types import SimpleNamespace
from unittest.mock import patch
from urllib.parse import parse_qs, urlparse
from urllib.request import Request

import requests
import responses

import cli


_ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")
_COMMIT_URL_RE = re.compile(
    r"https://cloud\.culturedcode\.com/version/1/history/[^/]+/commit\?ancestor-index=\d+&_cnt=1"
)


@dataclass
class CapturedCommit:
    url: str
    ancestor_index: int
    payload: dict


@dataclass
class MutatingRunResult:
    stdout: str
    stderr: str
    commits: list[CapturedCommit]


class _URLLibResponse:
    def __init__(self, content: bytes):
        self._content = content

    def read(self) -> bytes:
        return self._content

    def __enter__(self) -> "_URLLibResponse":
        return self

    def __exit__(self, exc_type, exc, tb) -> bool:
        return False


def _bridge_urlopen(req: Request) -> _URLLibResponse:
    response = requests.request(
        req.get_method(),
        req.full_url,
        data=req.data,
        headers=dict(req.header_items()),
    )
    return _URLLibResponse(response.content)


def run_cli_mutating_http(
    args: str,
    store,
    *,
    history_key: str = "A7h5eCi24RvAWKC3Hv3muf",
    initial_head_index: int = 100,
    server_head_indexes: list[int] | None = None,
    extra_patches: list[SimpleNamespace] | None = None,
) -> MutatingRunResult:
    argv = ["cli.py", "--no-color", *shlex.split(args)]
    stdout = io.StringIO()
    stderr = io.StringIO()
    commits: list[CapturedCommit] = []
    response_indexes = list(server_head_indexes or [])

    def _seed_state(client):
        client.history_key = history_key
        client.head_index = initial_head_index
        return {}

    def _on_commit(req):
        parsed = urlparse(req.url)
        ancestor_raw = parse_qs(parsed.query).get("ancestor-index", ["0"])[0]
        body = req.body
        if isinstance(body, bytes):
            body = body.decode("utf-8")
        payload = json.loads(body) if body else {}
        ancestor_index = int(ancestor_raw)
        commits.append(
            CapturedCommit(
                url=req.url,
                ancestor_index=ancestor_index,
                payload=payload,
            )
        )

        if response_indexes:
            next_index = response_indexes.pop(0)
        else:
            next_index = ancestor_index + 1

        return (
            200,
            {"Content-Type": "application/json"},
            json.dumps({"server-head-index": next_index}),
        )

    with responses.RequestsMock(assert_all_requests_are_fired=False) as rsps:
        rsps.add_callback(
            responses.POST,
            _COMMIT_URL_RE,
            callback=_on_commit,
            content_type="application/json",
        )

        with ExitStack() as stack:
            stack.enter_context(
                patch("cli.load_auth", return_value=("test@example.com", "secret"))
            )
            stack.enter_context(
                patch("cli.get_state_with_append_log", side_effect=_seed_state)
            )
            stack.enter_context(
                patch("cli.fold_state_from_append_log", return_value={})
            )
            stack.enter_context(patch("cli.ThingsStore", return_value=store))
            stack.enter_context(
                patch("things_cloud.client.urlopen", side_effect=_bridge_urlopen)
            )
            stack.enter_context(patch("sys.argv", argv))
            if extra_patches:
                for p in extra_patches:
                    stack.enter_context(patch(p.target, **p.kwargs))

            with redirect_stdout(stdout), redirect_stderr(stderr):
                cli.main()

    return MutatingRunResult(
        stdout=_ANSI_RE.sub("", stdout.getvalue()),
        stderr=_ANSI_RE.sub("", stderr.getvalue()),
        commits=commits,
    )


def p(target: str, **kwargs) -> SimpleNamespace:
    return SimpleNamespace(target=target, kwargs=kwargs)
