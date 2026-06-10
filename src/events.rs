use crate::rules::RuleMatch;
use serde::Serialize;
use std::io;
use std::path::{Path, PathBuf};

/// Set this env var to "1" to make `dont doctor` emit rejection event files.
pub const EVENTS_ENV: &str = "DONT_EMIT_EVENTS";
/// Subdirectory of `.dont/` where event files are written.
pub const EVENTS_SUBDIR: &str = "events";

#[derive(Debug, Serialize, PartialEq)]
pub struct ViolationRecord {
    pub entity_id: String,
    pub detail: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct RejectionEvent {
    pub schema_version: String,
    pub tool: String,
    pub event_kind: String,
    pub rule_name: String,
    pub timestamp: String,
    pub violations: Vec<ViolationRecord>,
}

/// Write a rejection event file to `<dont_dir>/events/` when `DONT_EMIT_EVENTS=1`.
///
/// Returns `Ok(None)` when the env var is absent (normal case — no side effects).
/// Returns `Ok(Some(path))` on success, `Err` on I/O failure.
pub fn emit_if_enabled(
    dont_dir: &Path,
    rule_name: &str,
    matches: &[RuleMatch],
) -> io::Result<Option<PathBuf>> {
    if std::env::var(EVENTS_ENV).is_err() {
        return Ok(None);
    }
    emit(dont_dir, rule_name, matches).map(Some)
}

/// Write a rejection event file unconditionally (no env-var check).
/// Callers that want the feature-flag guard should use [`emit_if_enabled`].
pub fn emit(dont_dir: &Path, rule_name: &str, matches: &[RuleMatch]) -> io::Result<PathBuf> {
    let events_dir = dont_dir.join(EVENTS_SUBDIR);
    std::fs::create_dir_all(&events_dir)?;
    let ts = now_rfc3339();
    let slug: String = ts
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let path = events_dir.join(format!("rejection-{rule_name}-{slug}.json"));
    let event = RejectionEvent {
        schema_version: "1.0".to_string(),
        tool: "dont".to_string(),
        event_kind: "rule_rejection".to_string(),
        rule_name: rule_name.to_string(),
        timestamp: ts,
        violations: matches
            .iter()
            .map(|m| ViolationRecord {
                entity_id: m.entity_id.clone(),
                detail: m.detail.clone(),
            })
            .collect(),
    };
    let json = serde_json::to_string_pretty(&event).map_err(io::Error::other)?;
    std::fs::write(&path, json)?;
    Ok(path)
}

fn now_rfc3339() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let (y, mo, d, h, mi, s) = epoch_to_parts(secs);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

fn is_leap(y: u64) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}

fn days_in_month(year: u64, month: u8) -> u64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn epoch_to_parts(secs: u64) -> (u64, u8, u8, u8, u8, u8) {
    let sec = (secs % 60) as u8;
    let mins = secs / 60;
    let min = (mins % 60) as u8;
    let hours = mins / 60;
    let hour = (hours % 24) as u8;
    let mut days = hours / 24;
    let mut year = 1970u64;
    loop {
        let yd = if is_leap(year) { 366 } else { 365 };
        if days < yd {
            break;
        }
        days -= yd;
        year += 1;
    }
    let mut month = 1u8;
    loop {
        let dim = days_in_month(year, month);
        if days < dim {
            break;
        }
        days -= dim;
        month += 1;
    }
    (year, month, (days + 1) as u8, hour, min, sec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_match(entity_id: &str, detail: &str) -> RuleMatch {
        RuleMatch {
            entity_id: entity_id.to_string(),
            detail: detail.to_string(),
        }
    }

    #[test]
    fn emit_creates_event_file() {
        let dir = TempDir::new().unwrap();
        let path = emit(
            dir.path(),
            "ungrounded",
            &[make_match(
                "claim:abc",
                "depends on unresolved CURIE 'foo:bar'",
            )],
        )
        .unwrap();
        assert!(path.exists());
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(v["schema_version"], "1.0");
        assert_eq!(v["tool"], "dont");
        assert_eq!(v["event_kind"], "rule_rejection");
        assert_eq!(v["rule_name"], "ungrounded");
        assert_eq!(v["violations"][0]["entity_id"], "claim:abc");
    }

    #[test]
    fn emit_creates_events_dir_if_missing() {
        let dir = TempDir::new().unwrap();
        let _ = emit(dir.path(), "ungrounded", &[]).unwrap();
        assert!(dir.path().join("events").is_dir());
    }

    #[test]
    fn emit_empty_violations_produces_valid_json() {
        let dir = TempDir::new().unwrap();
        let path = emit(dir.path(), "ungrounded", &[]).unwrap();
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(v["violations"].as_array().unwrap().is_empty());
    }

    #[test]
    fn emit_if_enabled_returns_none_without_env() {
        // Relies on DONT_EMIT_EVENTS not being set in the test environment.
        // (Do not set it in CI without clearing it here.)
        let dir = TempDir::new().unwrap();
        if std::env::var(EVENTS_ENV).is_ok() {
            return; // skip if the var happens to be set externally
        }
        let result = emit_if_enabled(dir.path(), "ungrounded", &[]).unwrap();
        assert!(result.is_none());
        assert!(!dir.path().join("events").exists());
    }

    #[test]
    fn epoch_to_parts_unix_epoch() {
        let (y, mo, d, h, mi, s) = epoch_to_parts(0);
        assert_eq!((y, mo, d, h, mi, s), (1970, 1, 1, 0, 0, 0));
    }

    #[test]
    fn epoch_to_parts_known_date() {
        // 2026-06-10 12:00:00 UTC = 1781092800
        let (y, mo, d, h, _mi, _s) = epoch_to_parts(1781092800);
        assert_eq!(y, 2026);
        assert_eq!(mo, 6);
        assert_eq!(d, 10);
        assert_eq!(h, 12);
    }
}
