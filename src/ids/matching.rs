use crate::ids::ThingsId;
use std::collections::HashMap;

pub fn lcp_len(a: &str, b: &str) -> usize {
    let mut i = 0usize;
    let max = a.len().min(b.len());
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    while i < max && a_bytes[i] == b_bytes[i] {
        i += 1;
    }
    i
}

pub fn shortest_unique_prefixes(ids: &[ThingsId]) -> HashMap<ThingsId, String> {
    if ids.is_empty() {
        return HashMap::new();
    }

    let mut ordered = ids.to_vec();
    ordered.sort();

    let mut result = HashMap::new();
    for (i, value) in ordered.iter().enumerate() {
        let value_s = value.to_string();
        let left = if i > 0 {
            let prev = ordered[i - 1].to_string();
            lcp_len(&value_s, &prev)
        } else {
            0
        };
        let right = if i + 1 < ordered.len() {
            let next = ordered[i + 1].to_string();
            lcp_len(&value_s, &next)
        } else {
            0
        };
        let need = left.max(right) + 1;
        result.insert(value.clone(), value_s.chars().take(need).collect());
    }

    result
}

pub fn prefix_matches<'a>(sorted_ids: &'a [ThingsId], prefix: &str) -> Vec<&'a ThingsId> {
    sorted_ids
        .iter()
        .filter(|id| id.starts_with(prefix))
        .collect()
}
