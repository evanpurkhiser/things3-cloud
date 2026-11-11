# Demo GIF

This folder contains the scripted workflow used to create the README/demo GIF
for `things3`.

The scripts are intentionally written
for a disposable Things Cloud test account.
Do not run them against a real personal account: the setup script creates tasks,
projects, and checklist items, and the cleanup script removes them afterward.

## Requirements

- `things3` on `PATH`
- `asciinema` on `PATH`
- `agg` on `PATH` for GIF rendering
- A monospace font available to `agg`

On this machine the working setup was:

```bash
uv tool install asciinema
cargo install --git https://github.com/asciinema/agg
```

The Linux server did not have system fonts installed,
so user-local fonts were used.
Install both a monospace text font and a symbol fallback so the Things glyphs
and box drawing characters render correctly:

```bash
mkdir -p ~/.local/share/fonts
curl -L --fail \
  -o ~/.local/share/fonts/NotoSansMono-Regular.ttf \
  'https://github.com/notofonts/noto-fonts/raw/main/hinted/ttf/NotoSansMono/NotoSansMono-Regular.ttf'
curl -L --fail \
  -o ~/.local/share/fonts/NotoSansSymbols2-Regular.ttf \
  'https://github.com/notofonts/noto-fonts/raw/main/hinted/ttf/NotoSansSymbols2/NotoSansSymbols2-Regular.ttf'
```

macOS generally renders nicer because `agg` can see the system font catalog.
The render script defaults to a larger `22px` font size, `1.4` line height,
and separate text and emoji font families:

```bash
DEMO_FONT_SIZE=24 \
DEMO_LINE_HEIGHT=1.4 \
DEMO_TEXT_FONT_FAMILY='Cascadia Mono,SF Mono,Noto Sans Symbols 2' \
DEMO_EMOJI_FONT_FAMILY='Apple Color Emoji,Noto Emoji,Noto Color Emoji' \
tools/demo-gif/render.sh
```

Use `DEMO_AGG_BIN=/path/to/agg` to render with a locally patched `agg` binary.

## Chosen README Themes

The README uses two rendered GIFs in `.github/assets`:

- Light mode: a custom `asciinema-light`-style palette with darker green
  for better contrast on white backgrounds.
- Dark mode: the built-in `asciinema` theme.

The selected light theme string is:

```text
f7f7f7,24292f,24292f,dd3c69,22863a,9a6700,0969da,b954e1,0a7f70,57606a,6e7781,cf222e,1a7f37,9a6700,0969da,8250df,0a7f70,24292f
```

Render the README assets from a finished cast with:

```bash
DEMO_AGG_BIN=/path/to/patched/agg \
DEMO_TEXT_FONT_FAMILY='Cascadia Mono,Noto Sans Symbols 2' \
DEMO_LINE_HEIGHT=1.4 \
DEMO_THEME='f7f7f7,24292f,24292f,dd3c69,22863a,9a6700,0969da,b954e1,0a7f70,57606a,6e7781,cf222e,1a7f37,9a6700,0969da,8250df,0a7f70,24292f' \
tools/demo-gif/render.sh /tmp/things3-demo-gif/things3-cli-demo.cast .github/assets/demo-light.gif

DEMO_AGG_BIN=/path/to/patched/agg \
DEMO_TEXT_FONT_FAMILY='Cascadia Mono,Noto Sans Symbols 2' \
DEMO_LINE_HEIGHT=1.4 \
DEMO_THEME=asciinema \
tools/demo-gif/render.sh /tmp/things3-demo-gif/things3-cli-demo.cast .github/assets/demo-dark.gif
```

## Credentials

Credentials are read from environment variables only.
Do not commit credentials or an auth file.

```bash
export THINGS3_EMAIL='test-account@example.com'
export THINGS3_PASSWORD='test-account-password'
```

## Full Workflow

```bash
tools/demo-gif/run.sh
```

By default outputs are written to:

- `/tmp/things3-demo-gif/things3-cli-demo.cast`
- `/tmp/things3-demo-gif/things3-cli-demo.gif`

Override the output/work directory with:

```bash
DEMO_WORK_DIR=/tmp/my-demo tools/demo-gif/run.sh
```

## Individual Steps

```bash
tools/demo-gif/setup.sh
tools/demo-gif/record.sh
tools/demo-gif/render.sh
tools/demo-gif/cleanup.sh
```

`setup.sh` stages a realistic test account state.
`record.sh` records a natural terminal flow that uses short IDs,
including listing a project in detailed mode before checking checklist items
so the IDs are visible.
`cleanup.sh` removes checklist items
before deleting their parent tasks to avoid orphaned checklist records.

Commands are typed out during recording by default.
Disable typing with `DEMO_TYPE_COMMANDS=0`,
or tune the per-character delay with `DEMO_TYPE_DELAY=0.025`.

## Current Demo Flow

The visible recording does this:

```bash
things3 today
things3 mark <short-id> --done
things3 new "Call dentist about night guard" --when today --notes "..."
clear
things3 today --detailed
clear
things3 projects list
things3 schedule <project-short-id> --when today
things3 project <project-short-id>
things3 edit <task-short-id> --add-checklist "send building availability"
things3 project <project-short-id> --detailed
things3 mark <task-short-id> --check <checklist-short-id>
things3 mark <task-short-id> --check <checklist-short-id>
things3 project <project-short-id> --detailed
```

The prompt is rendered as a gray `$` and command text is left
as normal terminal foreground, matching a more natural terminal recording.
