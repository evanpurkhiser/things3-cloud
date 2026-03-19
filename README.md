# things3

[![Tests](https://github.com/evanpurkhiser/things3-cli/actions/workflows/tests.yml/badge.svg)](https://github.com/evanpurkhiser/things3-cli/actions/workflows/tests.yml)

> [!NOTE]
> This project is written completely using Claude. No review of the code has been done.

A command-line interface for [Things 3](https://culturedcode.com/things/) that communicates
directly with the Things Cloud API — keeping all your tasks in sync across your Apple devices.

## Quick Start

Install from GitHub:

```bash
uv tool install "git+https://github.com/evanpurkhiser/things3-cli"
```

Authenticate with your Things Cloud credentials:

```
$ things3 set-auth
Things Cloud email: you@example.com
Things Cloud password: ••••••••••••
```

Then run commands:

```
$ things3 today
⭑ Today  (9 tasks)

  LZ4 ▢ Follow up with team
      │ Shared notes for context
      │
    D ├╴○ draft update
    J └╴○ review checklist
  4HY ▢ Task with notes
      │ Multi-line note example
      └ with a second line
  Uuq ▢ Review inbox and prioritize
  699 ▢ Prepare weekly summary  [Planning]
  H47 ▢ Submit reimbursement  ⚑ due by 2026-04-10

  ☽ This Evening
  8KU ▢ Reflect on highlights
  63E ▢ Reset workspace for tomorrow
```

## Install

Install as a `uv` tool:

```bash
uv tool install .
```

Install from GitHub:

```bash
uv tool install "git+https://github.com/evanpurkhiser/things3-cli"
```

## Try with uvx

Run directly from GitHub without installing:

```bash
uvx --from "git+https://github.com/evanpurkhiser/things3-cli" things3 --help
```

## Supported features

- [x] Configure auth with `set-auth` (stored in XDG state)
- [x] Incremental sync with local cache for fast startup
- [x] `--no-sync` flag to skip cloud sync and use local cache only

**Views**
- [x] `inbox`, `today`, `upcoming`, `anytime`, `someday`, `logbook`
- [x] `projects` / `project <id>`, `areas` / `area <id>`, `tags`
- [x] Show notes via `--detailed` flag
- [x] Show checklist items via `--detailed`
- [x] `tags` renders nested tag hierarchy with box-drawing tree
- [x] `find` — search by title/notes/checklists, filter by tag, area, project, status, date ranges
- [ ] Machine-readable output (`--json`, `--toon`) for scripting and LLM/tool use

**Tasks**
- [x] `new` — create tasks with title, notes, tags, when, deadline, position
- [x] `edit` — rename, set/remove notes, move, add/remove tags (supports multiple IDs)
- [x] `edit --add-checklist/--remove-checklist/--rename-checklist` — manage checklist items
- [x] `mark` — set status to done/incomplete/canceled (supports multiple IDs)
- [x] `mark --check/--uncheck/--check-cancel` — toggle checklist items by short ID
- [x] `schedule` — set when/start date, deadline, today/evening/someday
- [x] `reorder` — reorder within lists
- [x] `delete` — trash tasks
- [ ] Set/remove recurrence via `edit`

**Projects**
- [x] `projects new` — create projects with title, notes, tags, when, deadline, area
- [x] `projects edit` — edit title, notes, area, add/remove tags
- [ ] Heading management — create/rename/delete/reorder headings within projects

**Areas**
- [x] `areas new` — create areas with title and tags
- [x] `areas edit` — edit title, add/remove tags

**Tags**
- [x] `tags new` — create tags with optional `--parent` for nesting
- [x] `tags edit` — rename (`--name`) or reparent (`--move`, supports `clear`)
- [x] `tags delete` — delete a tag by title or UUID prefix

**Testing**
- [ ] Sync engine tests (append log replay, incremental fold, state caching)
- [ ] Things store tests (task queries, project progress, prefix resolution)
- [x] End-to-end integration tests for read + mutating command flows

## Dev

```bash
uv run python -m py_compile cli.py things_cloud/client.py things_cloud/store.py
```
