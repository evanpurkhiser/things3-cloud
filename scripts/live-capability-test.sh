#!/usr/bin/env bash
set -Eeuo pipefail

# Destructive live integration test for a throwaway Things Cloud account.
#
# This intentionally writes to Things Cloud. It must never run implicitly from CI.
# Use a dedicated test account and opt in explicitly:
#
#   THINGS3_LIVE_TEST=1 THINGS3_BIN=./target/debug/things3 scripts/live-capability-test.sh

if [[ "${THINGS3_LIVE_TEST:-}" != "1" ]]; then
	printf 'Refusing to run live test without THINGS3_LIVE_TEST=1\n' >&2
	exit 2
fi

if ! command -v jq >/dev/null 2>&1; then
	printf 'jq is required for live capability tests\n' >&2
	exit 2
fi

THINGS3_BIN="${THINGS3_BIN:-things3}"
ACTION_DELAY="${THINGS3_LIVE_TEST_DELAY:-0.25}"
RUN_ID="${THINGS3_LIVE_TEST_RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$$}"
PREFIX="${THINGS3_LIVE_TEST_PREFIX:-things3-cloud-live-test-${RUN_ID}}"
FUTURE_DATE="${THINGS3_LIVE_TEST_FUTURE_DATE:-$(python3 - <<'PY'
from datetime import date, timedelta
print(date.today() + timedelta(days=14))
PY
)}"

projects=()
tasks=()
areas=()
tags=()

log() {
	printf '\n==> %s\n' "$*"
}

pause_between_actions() {
	if [[ "$ACTION_DELAY" != "0" && "$ACTION_DELAY" != "0.0" ]]; then
		sleep "$ACTION_DELAY"
	fi
}

run() {
	printf '+ %q' "$THINGS3_BIN" >&2
	printf ' %q' --no-color "$@" >&2
	printf '\n' >&2
	"$THINGS3_BIN" --no-color "$@"
	pause_between_actions
}

capture_created_id() {
	local output id
	output="$(run "$@")"
	printf '%s\n' "$output" >&2
	id="$(sed -n 's/.*(\([[:alnum:]]\{20,\}\)).*/\1/p' <<<"$output" | tail -n1)"

	if [[ -z "$id" ]]; then
		id="$(awk 'NF {print $NF}' <<<"$output" | tail -n1)"
	fi

	if [[ ! "$id" =~ ^[[:alnum:]]{20,}$ ]]; then
		printf 'Could not parse created id from output for: %s\n' "$*" >&2
		exit 1
	fi

	printf '%s\n' "$id"
}

create_tag() {
	local -n target=$1
	shift
	target="$(capture_created_id tags new "$@")"
	tags+=("$target")
}

create_area() {
	local -n target=$1
	shift
	target="$(capture_created_id areas new "$@")"
	areas+=("$target")
}

create_project() {
	local -n target=$1
	shift
	target="$(capture_created_id projects new "$@")"
	projects+=("$target")
}

create_task() {
	local -n target=$1
	shift
	target="$(capture_created_id new "$@")"
	tasks+=("$target")
}

cleanup() {
	local status=$? i
	set +e

	printf '\n==> cleanup\n'

	for ((i = ${#tasks[@]} - 1; i >= 0; i--)); do
		[[ -n "${tasks[i]}" ]] && "$THINGS3_BIN" --no-color delete "${tasks[i]}" >/dev/null 2>&1
	done

	for ((i = ${#projects[@]} - 1; i >= 0; i--)); do
		[[ -n "${projects[i]}" ]] && "$THINGS3_BIN" --no-color delete "${projects[i]}" >/dev/null 2>&1
	done

	for ((i = ${#areas[@]} - 1; i >= 0; i--)); do
		[[ -n "${areas[i]}" ]] && "$THINGS3_BIN" --no-color delete "${areas[i]}" >/dev/null 2>&1
	done

	for ((i = ${#tags[@]} - 1; i >= 0; i--)); do
		[[ -n "${tags[i]}" ]] && "$THINGS3_BIN" --no-color tags delete "${tags[i]}" >/dev/null 2>&1
	done

	if [[ $status -eq 0 ]]; then
		printf 'cleanup complete\n'
	else
		printf 'cleanup attempted after failure (exit %s)\n' "$status" >&2
	fi

	exit "$status"
}

trap cleanup EXIT

json_project() {
	"$THINGS3_BIN" --no-color --json project "$1"
}

first_checklist_id() {
	local project_id=$1 task_id=$2
	json_project "$project_id" | jq -r --arg task_id "$task_id" '
		.[] | select(.id == $task_id) | .checklist[0].id // empty
	'
}

log "preflight read"
run today >/dev/null

log "tag management"
create_tag tag_parent "${PREFIX} tag parent"
create_tag tag_child "${PREFIX} tag child" --parent "$tag_parent"
run tags edit "$tag_child" --name "${PREFIX} tag child renamed"
run tags edit "$tag_child" --move clear
run tags edit "$tag_child" --move "$tag_parent"

log "area management"
create_area area_id "${PREFIX} area" --tags "$tag_child"
run areas edit "$area_id" --title "${PREFIX} area renamed"
run areas edit "$area_id" --add-tags "$tag_parent"
run areas edit "$area_id" --remove-tags "$tag_parent"

log "project management"
create_project project_id "${PREFIX} project" --area "$area_id" --tags "$tag_child" --notes "Project notes" --deadline "$FUTURE_DATE"
create_project project_today "${PREFIX} project today" --when today
create_project project_someday "${PREFIX} project someday" --when someday
create_project project_future "${PREFIX} project future" --when "$FUTURE_DATE" --deadline "$FUTURE_DATE"
run projects edit "$project_id" --title "${PREFIX} project renamed"
run projects edit "$project_id" --notes "Project notes updated"
run projects edit "$project_id" --notes ""
run projects edit "$project_id" --add-tags "$tag_parent"
run projects edit "$project_id" --remove-tags "$tag_parent"
run projects edit "$project_id" --remove-tags "$tag_child"
run projects edit "$project_id" --move clear
run projects edit "$project_id" --move "$area_id"
run schedule "$project_id" --clear-deadline
run schedule "$project_future" --clear-deadline

log "project task creation variants"
create_task project_task "${PREFIX} project task" --in "$project_id"
create_task project_today "${PREFIX} project today" --in "$project_id" --when today
create_task project_future "${PREFIX} project future" --in "$project_id" --when "$FUTURE_DATE" --deadline "$FUTURE_DATE"
create_task project_someday "${PREFIX} project someday" --in "$project_id" --when someday
create_task project_tagged "${PREFIX} project tagged" --in "$project_id" --tags "$tag_child" --notes "Tagged task notes"
create_task project_checklist "${PREFIX} project checklist" --in "$project_id"

log "root task creation variants"
create_task inbox_task "${PREFIX} inbox task"
create_task anytime_task "${PREFIX} anytime task" --when anytime
create_task today_task "${PREFIX} today task" --when today
create_task future_task "${PREFIX} future task" --when "$FUTURE_DATE" --deadline "$FUTURE_DATE"
create_task someday_task "${PREFIX} someday task" --when someday
create_task area_task "${PREFIX} area task" --in "$area_id"
create_task tagged_task "${PREFIX} tagged task" --tags "$tag_child" --notes "Tagged root task notes"

log "task editing and movement"
run edit "$inbox_task" --title "${PREFIX} inbox task renamed"
run edit "$inbox_task" --notes "Updated inbox notes"
run edit "$inbox_task" --add-tags "$tag_child"
run edit "$inbox_task" --remove-tags "$tag_child"
run edit "$inbox_task" --move "$project_id"
run edit "$inbox_task" --move "$area_id"
run edit "$inbox_task" --move inbox

log "scheduling"
run schedule "$anytime_task" --when today
run schedule "$anytime_task" --when evening
run schedule "$anytime_task" --when someday
run schedule "$anytime_task" --when "$FUTURE_DATE" --deadline "$FUTURE_DATE"
run schedule "$anytime_task" --when anytime --clear-deadline

log "checklist mutations"
run edit "$project_checklist" --add-checklist "First checklist item" --add-checklist "Second checklist item"
checklist_id="$(first_checklist_id "$project_id" "$project_checklist")"

if [[ -z "$checklist_id" ]]; then
	printf 'Could not discover checklist id for task %s\n' "$project_checklist" >&2
	exit 1
fi

run mark "$project_checklist" --check "$checklist_id"
run mark "$project_checklist" --uncheck "$checklist_id"
run mark "$project_checklist" --check-cancel "$checklist_id"
run edit "$project_checklist" --rename-checklist "${checklist_id}:${PREFIX} renamed checklist item"
run edit "$project_checklist" --remove-checklist "$checklist_id"

log "status transitions"
run mark "$project_task" --done
run mark "$project_task" --incomplete
run mark "$project_task" --canceled

log "reordering"
run reorder "$project_future" --after-id "$project_today"
run reorder "$future_task" --after-id "$someday_task"

log "read views"
run inbox >/dev/null
run anytime >/dev/null
run someday >/dev/null
run upcoming >/dev/null
run today >/dev/null
run projects >/dev/null
run areas >/dev/null
run tags >/dev/null
run project "$project_id" >/dev/null
run area "$area_id" >/dev/null
run find "$PREFIX" --any-status >/dev/null

log "live capability test completed"
