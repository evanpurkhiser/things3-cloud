use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use things_cli::wire::wire_object::{Properties, WireItem};

fn default_journal_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/evan".to_string());
    PathBuf::from(home).join(".local/state/things3/append-log/things.log")
}

fn allowed_keys(entity: &str) -> Option<&'static [&'static str]> {
    match entity {
        "Task6" => Some(&[
            "acrd", "agr", "ar", "ato", "cd", "dd", "dds", "dl", "do", "icc", "icp", "icsd", "ix",
            "lai", "lt", "md", "nt", "pr", "rmd", "rp", "rr", "rt", "sb", "sp", "sr", "ss", "st",
            "tg", "ti", "tir", "tp", "tr", "tt", "xx",
        ]),
        "ChecklistItem3" => Some(&["cd", "ix", "lt", "md", "sp", "ss", "ts", "tt", "xx"]),
        "Tag4" => Some(&["ix", "md", "pn", "sh", "tt", "xx"]),
        "Area3" => Some(&["cd", "ix", "md", "tg", "tt", "xx"]),
        "Tombstone2" => Some(&["dld", "dloid"]),
        "Command" => Some(&["cd", "if", "tp"]),
        _ => None,
    }
}

#[test]
#[ignore = "manual helper for local journal coverage"]
fn journal_has_no_unknown_keys_for_current_wire_entities() {
    let path = std::env::var("THINGS_JOURNAL_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_journal_path());

    let file = File::open(&path).expect("open journal file");
    let reader = BufReader::new(file);

    let mut unknown_keys_by_entity: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut unknown_payload_for_current_entities = 0usize;

    for line in reader.lines() {
        let line = line.expect("read journal line");
        if line.trim().is_empty() {
            continue;
        }

        let raw: Value = serde_json::from_str(&line).expect("line parses as json object");
        let raw_item = raw.as_object().expect("line is top-level object");

        for obj in raw_item.values() {
            let wire_obj = obj.as_object().expect("wire object is map");
            let entity = wire_obj.get("e").and_then(Value::as_str).unwrap_or("");
            let op = wire_obj.get("t").and_then(Value::as_i64).unwrap_or(-1);
            let props = wire_obj
                .get("p")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();

            if let Some(allowed) = allowed_keys(entity) {
                if op == 0 || op == 1 {
                    let allowed: BTreeSet<&str> = allowed.iter().copied().collect();
                    for key in props.keys() {
                        if !allowed.contains(key.as_str()) {
                            unknown_keys_by_entity
                                .entry(entity.to_string())
                                .or_default()
                                .insert(key.to_string());
                        }
                    }
                }
            }
        }

        let parsed: WireItem = serde_json::from_str(&line).expect("line deserializes to WireItem");
        for obj in parsed.values() {
            let entity = obj
                .entity_type
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default();
            let is_current = matches!(
                entity.as_str(),
                "Task6" | "ChecklistItem3" | "Tag4" | "Area3" | "Tombstone2" | "Command"
            );
            if is_current {
                let props = obj.properties().expect("typed properties");
                if matches!(props, Properties::Unknown(_)) {
                    unknown_payload_for_current_entities += 1;
                }
            }
        }
    }

    assert!(
        unknown_keys_by_entity.is_empty(),
        "unknown keys detected for current entities: {:?}",
        unknown_keys_by_entity
    );
    assert_eq!(
        unknown_payload_for_current_entities, 0,
        "current entities decoded into Properties::Unknown"
    );
}
