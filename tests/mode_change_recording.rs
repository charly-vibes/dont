mod common;

use common::{conclude_claim, dont, init_dir};
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

fn events_jsonl(dir: &TempDir) -> Vec<Value> {
    // When DONT_DIR is set, files are directly inside that directory (no .dont subdir).
    let path = dir.path().join("events.jsonl");
    let text = fs::read_to_string(path).unwrap_or_default();
    text.lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

fn switch_mode(dir: &TempDir, from: &str, to: &str) {
    let config_path = dir.path().join("config.toml");
    let original = fs::read_to_string(&config_path).expect("config.toml must exist");
    let updated = original.replace(&format!("mode = \"{from}\""), &format!("mode = \"{to}\""));
    fs::write(&config_path, updated).expect("could not write config.toml");
}

// --- init records mode in events.jsonl ---

#[test]
fn init_writes_mode_to_events() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let events = events_jsonl(&dir);
    // The project.initialized event already contains mode; subsequent commands
    // must not write a spurious mode.changed if mode hasn't changed.
    let mode_event = events
        .iter()
        .find(|e| e["mode"].is_string())
        .expect("at least one event with a 'mode' field must exist after init");

    assert!(
        mode_event["mode"].is_string(),
        "init event must record the project mode; got: {mode_event}"
    );
}

#[test]
fn strict_init_records_strict_mode() {
    let dir = TempDir::new().unwrap();
    dont()
        .args(["init", "--strict"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let events = events_jsonl(&dir);
    let init_event = events
        .iter()
        .find(|e| e["kind"] == "project.initialized")
        .expect("a project.initialized event must exist");

    assert_eq!(
        init_event["mode"].as_str().unwrap(),
        "strict",
        "project.initialized must record mode=strict; got: {init_event}"
    );
}

#[test]
fn no_spurious_mode_changed_when_mode_unchanged() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // invoke multiple times without switching mode
    conclude_claim(&dir, "first");
    conclude_claim(&dir, "second");

    let events = events_jsonl(&dir);
    let changed_count = events
        .iter()
        .filter(|e| e["kind"] == "mode.changed")
        .count();

    assert_eq!(
        changed_count, 0,
        "no mode.changed event must be written when mode has not changed; got {changed_count}"
    );
}

// --- mode.changed is written when config.toml mode changes ---

#[test]
fn mode_change_writes_mode_changed_event() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // establish baseline
    conclude_claim(&dir, "seed claim");

    // change mode in config.toml
    switch_mode(&dir, "permissive", "strict");

    // trigger check
    conclude_claim(&dir, "post-switch claim");

    let events = events_jsonl(&dir);
    let change = events
        .iter()
        .find(|e| e["kind"] == "mode.changed")
        .expect("a mode.changed event must be written after config mode switch");

    assert_eq!(
        change["from"].as_str().unwrap(),
        "permissive",
        "mode.changed.from must be the old mode"
    );
    assert_eq!(
        change["to"].as_str().unwrap(),
        "strict",
        "mode.changed.to must be the new mode"
    );
    assert_eq!(
        change["mode"].as_str().unwrap(),
        "strict",
        "mode.changed.mode must be the current mode"
    );
    assert!(
        change["created_at"].is_string(),
        "mode.changed must include created_at"
    );
}

// --- subsequent reads reflect new mode ---

#[test]
fn prime_reports_new_mode_after_switch() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    switch_mode(&dir, "permissive", "strict");

    let out = dont()
        .args(["prime", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    let mode = v["data"]["mode"]
        .as_str()
        .expect("data.mode must be present");
    assert_eq!(
        mode, "strict",
        "prime must report the new mode after switch"
    );
}

// --- multiple transitions produce ordered history ---

#[test]
fn multiple_mode_transitions_produce_ordered_history() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    conclude_claim(&dir, "seed");
    switch_mode(&dir, "permissive", "strict");
    conclude_claim(&dir, "post strict");
    switch_mode(&dir, "strict", "permissive");
    conclude_claim(&dir, "post permissive");

    let events = events_jsonl(&dir);
    let mode_events: Vec<&Value> = events
        .iter()
        .filter(|e| {
            e["kind"] == "mode.changed" || e["kind"] == "mode.baseline" || e["kind"] == "mode.init"
        })
        .collect();

    // must have at least: baseline + two mode.changed events
    let changed: Vec<&&Value> = mode_events
        .iter()
        .filter(|e| e["kind"] == "mode.changed")
        .collect();

    assert!(
        changed.len() >= 2,
        "two mode switches must produce at least 2 mode.changed events; got {} changed events",
        changed.len()
    );
}

// --- mode.baseline is written for legacy projects that have no mode in events ---

#[test]
fn mode_baseline_written_for_projects_without_mode_in_events() {
    // Simulate a legacy project by removing mode from the initialized event.
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    // Strip the `mode` field from all events to simulate a pre-mode-tracking project.
    let events_path = dir.path().join("events.jsonl");
    let text = std::fs::read_to_string(&events_path).unwrap();
    let stripped: String = text
        .lines()
        .map(|line| {
            if let Ok(mut v) = serde_json::from_str::<serde_json::Value>(line) {
                v.as_object_mut().map(|o| o.remove("mode"));
                format!("{}\n", serde_json::to_string(&v).unwrap())
            } else {
                format!("{line}\n")
            }
        })
        .collect();
    std::fs::write(&events_path, stripped).unwrap();

    // Trigger check_and_record_mode_change
    conclude_claim(&dir, "trigger baseline");

    let events = events_jsonl(&dir);
    let baselines: Vec<&Value> = events
        .iter()
        .filter(|e| e["kind"] == "mode.baseline")
        .collect();

    assert_eq!(
        baselines.len(),
        1,
        "mode.baseline must be written exactly once for a legacy project; got {} baselines",
        baselines.len()
    );
}
