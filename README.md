# things3

A small Things 3 CLI client backed by Things Cloud.

## Supported features

- [x] Configure auth with `set-auth` (stored in XDG state)
- [x] Show lists: `today`, `anytime`, `inbox`, `upcoming`
- [x] List projects with `projects`
- [x] List areas with `areas`
- [x] List tags with `tags`
- [x] Update task status with `mark --done|--incomplete|--canceled`
- [x] Replay cloud history (`t=0/1/2`) with append-only local cache in XDG state
- [ ] Show `someday` list and special lists (`tomorrow`, `deadlines`, `repeating`)
- [ ] Create tasks from CLI (`add`) with list/date/project/tag targeting
- [ ] Edit/move tasks (`schedule`, `tag`, `move`) for inbox processing workflows
- [ ] Add `find` / filters (tag, area, project, status) for fast retrieval
- [ ] Add machine-readable output (`--json`) optimized for LLM/tool use

## Quick Start

Set auth interactively:

```bash
things3 set-auth
```

Run commands:

```bash
things3 today
things3 inbox
things3 projects
things3 mark <task-id> --done
```

Install as a `uv` tool:

```bash
uv tool install .
```

## Dev

```bash
uv run python -m py_compile cli.py things_cloud/client.py things_cloud/store.py
```
