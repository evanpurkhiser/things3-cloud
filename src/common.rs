use crate::ids::ThingsId;
use crate::store::{Tag, ThingsStore};
use crate::wire::notes::{StructuredTaskNotes, TaskNotes};
use chrono::{DateTime, FixedOffset, Local, NaiveDate, TimeZone, Utc};
use crc32fast::Hasher;
use std::collections::HashSet;

/// Return today as a UTC midnight `DateTime<Utc>`.
pub fn today_utc() -> DateTime<Utc> {
    let today = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
    Utc.from_utc_datetime(&today)
}

/// Return current wall-clock unix timestamp in seconds (fractional).
pub fn now_ts_f64() -> f64 {
    Utc::now().timestamp_millis() as f64 / 1000.0
}

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const CYAN: &str = "\x1b[36m";
pub const YELLOW: &str = "\x1b[33m";
pub const GREEN: &str = "\x1b[32m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const RED: &str = "\x1b[31m";

#[derive(Debug, Clone, Copy)]
pub struct Icons {
    pub task_open: &'static str,
    pub task_done: &'static str,
    pub task_someday: &'static str,
    pub task_canceled: &'static str,
    pub evening: &'static str,
    pub today: &'static str,
    pub today_staged: &'static str,
    pub project: &'static str,
    pub area: &'static str,
    pub tag: &'static str,
    pub inbox: &'static str,
    pub anytime: &'static str,
    pub upcoming: &'static str,
    pub progress_empty: &'static str,
    pub progress_quarter: &'static str,
    pub progress_half: &'static str,
    pub progress_three_quarter: &'static str,
    pub progress_full: &'static str,
    pub deadline: &'static str,
    pub done: &'static str,
    pub incomplete: &'static str,
    pub canceled: &'static str,
    pub deleted: &'static str,
    pub checklist_open: &'static str,
    pub checklist_done: &'static str,
    pub checklist_canceled: &'static str,
    pub separator: &'static str,
    pub divider: &'static str,
}

pub const ICONS: Icons = Icons {
    task_open: "▢",
    task_done: "◼",
    task_someday: "⬚",
    task_canceled: "☒",
    evening: "☽",
    today: "⭑",
    today_staged: "●",
    project: "●",
    area: "◆",
    tag: "⌗",
    inbox: "⬓",
    anytime: "◌",
    upcoming: "▷",
    progress_empty: "◯",
    progress_quarter: "◔",
    progress_half: "◑",
    progress_three_quarter: "◕",
    progress_full: "◉",
    deadline: "⚑",
    done: "✓",
    incomplete: "↺",
    canceled: "☒",
    deleted: "×",
    checklist_open: "○",
    checklist_done: "●",
    checklist_canceled: "×",
    separator: "·",
    divider: "─",
};

pub fn colored<T: ToString>(text: T, codes: &[&str], no_color: bool) -> String {
    let text = text.to_string();
    if no_color {
        return text;
    }
    let mut out = String::new();
    for code in codes {
        out.push_str(code);
    }
    out.push_str(&text);
    out.push_str(RESET);
    out
}

pub fn fmt_date(dt: Option<DateTime<Utc>>) -> String {
    dt.map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

pub fn fmt_date_local(dt: Option<DateTime<Utc>>) -> String {
    let fixed_local = fixed_local_offset();
    dt.map(|d| d.with_timezone(&fixed_local).format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

fn fixed_local_offset() -> FixedOffset {
    let seconds = Local::now().offset().local_minus_utc();
    FixedOffset::east_opt(seconds).unwrap_or_else(|| FixedOffset::east_opt(0).expect("UTC offset"))
}

pub fn parse_day(day: Option<&str>, label: &str) -> Result<Option<DateTime<Local>>, String> {
    let Some(day) = day else {
        return Ok(None);
    };
    let parsed = NaiveDate::parse_from_str(day, "%Y-%m-%d")
        .map_err(|_| format!("Invalid {label} date: {day} (expected YYYY-MM-DD)"))?;
    let fixed_local = fixed_local_offset();
    let local_dt = parsed
        .and_hms_opt(0, 0, 0)
        .and_then(|d| fixed_local.from_local_datetime(&d).single())
        .map(|d| d.with_timezone(&Local))
        .ok_or_else(|| format!("Invalid {label} date: {day} (expected YYYY-MM-DD)"))?;
    Ok(Some(local_dt))
}

pub fn day_to_timestamp(day: DateTime<Local>) -> i64 {
    day.with_timezone(&Utc).timestamp()
}

pub fn task6_note(value: &str) -> TaskNotes {
    let mut hasher = Hasher::new();
    hasher.update(value.as_bytes());
    let checksum = hasher.finalize();
    TaskNotes::Structured(StructuredTaskNotes {
        object_type: Some("tx".to_string()),
        format_type: 1,
        ch: Some(checksum),
        v: Some(value.to_string()),
        ps: Vec::new(),
        unknown_fields: Default::default(),
    })
}

pub fn resolve_single_tag(store: &ThingsStore, identifier: &str) -> (Option<Tag>, String) {
    let identifier = identifier.trim();
    let all_tags = store.tags();

    let exact = all_tags
        .iter()
        .filter(|t| t.title.eq_ignore_ascii_case(identifier))
        .cloned()
        .collect::<Vec<_>>();
    if exact.len() == 1 {
        return (exact.first().cloned(), String::new());
    }
    if exact.len() > 1 {
        return (None, format!("Ambiguous tag title: {identifier}"));
    }

    let prefix = all_tags
        .iter()
        .filter(|t| t.uuid.starts_with(identifier))
        .cloned()
        .collect::<Vec<_>>();
    if prefix.len() == 1 {
        return (prefix.first().cloned(), String::new());
    }
    if prefix.len() > 1 {
        return (None, format!("Ambiguous tag UUID prefix: {identifier}"));
    }

    (None, format!("Tag not found: {identifier}"))
}

pub fn resolve_tag_ids(store: &ThingsStore, raw_tags: &str) -> (Vec<ThingsId>, String) {
    let tokens = raw_tags
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>();
    if tokens.is_empty() {
        return (Vec::new(), String::new());
    }

    let all_tags = store.tags();
    let mut resolved = Vec::new();
    let mut seen = HashSet::new();

    for token in tokens {
        let exact = all_tags
            .iter()
            .filter(|tag| tag.title.eq_ignore_ascii_case(token))
            .cloned()
            .collect::<Vec<_>>();

        if exact.len() == 1 {
            let tag_uuid = exact[0].uuid.clone();
            if seen.insert(tag_uuid.clone()) {
                resolved.push(tag_uuid);
            }
            continue;
        }
        if exact.len() > 1 {
            return (Vec::new(), format!("Ambiguous tag title: {token}"));
        }

        let prefix = all_tags
            .iter()
            .filter(|tag| tag.uuid.starts_with(token))
            .cloned()
            .collect::<Vec<_>>();

        if prefix.len() == 1 {
            let tag_uuid = prefix[0].uuid.clone();
            if seen.insert(tag_uuid.clone()) {
                resolved.push(tag_uuid);
            }
            continue;
        }
        if prefix.len() > 1 {
            return (Vec::new(), format!("Ambiguous tag UUID prefix: {token}"));
        }

        return (Vec::new(), format!("Tag not found: {token}"));
    }

    (resolved, String::new())
}
