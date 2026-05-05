use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use dont::store::{Store, StoreEvent};

#[test]
fn opens_project_store_at_canonical_path_and_records_schema_metadata() {
    let root = temp_project_root();

    let store = Store::open_project(&root).expect("store opens");

    assert!(root.join(".dont/db.cozo").exists());
    assert_eq!(store.schema_version().expect("schema version"), 1);
}

#[test]
fn append_event_persists_claim_datoms_with_monotonic_transactions() {
    let root = temp_project_root();
    let store = Store::open_project(&root).expect("store opens");

    let first = store
        .append_claim("unsupported assertions need grounding")
        .expect("first claim appends");
    let second = store
        .append_claim("sources should be explicit")
        .expect("second claim appends");

    assert!(first.id.starts_with("claim:"));
    assert!(first.event_id.starts_with("event:"));
    assert!(first.created_at.ends_with('Z'));
    assert!(!first.created_at.contains('.'));
    assert!(second.tx > first.tx);

    let loaded = store
        .claim_by_id(&first.id)
        .expect("query succeeds")
        .expect("claim exists");

    assert_eq!(loaded.id, first.id);
    assert_eq!(loaded.statement, "unsupported assertions need grounding");
    assert_eq!(loaded.status, "unverified");
    assert_eq!(loaded.events.len(), 1);
    assert_eq!(loaded.events[0].kind, "concluded");
    assert_eq!(loaded.events[0].tx, first.tx);
}

#[test]
fn status_changes_are_stored_as_retraction_and_assertion_datoms() {
    let root = temp_project_root();
    let store = Store::open_project(&root).expect("store opens");

    let claim = store.append_claim("truth needs pressure").expect("claim");
    let event = StoreEvent {
        kind: "trusted".to_string(),
        note: Some("source is ambiguous".to_string()),
    };

    let transition = store
        .append_status_change(&claim.id, "unverified", "doubted", event)
        .expect("status change appends");

    assert!(transition.tx > claim.tx);
    let datoms = store.datoms_for_entity(&claim.id).expect("datoms");
    assert!(datoms.iter().any(|d| {
        d.attribute == "status"
            && d.value == serde_json::json!("unverified")
            && d.tx == transition.tx
            && !d.assert_bit
    }));
    assert!(datoms.iter().any(|d| {
        d.attribute == "status"
            && d.value == serde_json::json!("doubted")
            && d.tx == transition.tx
            && d.assert_bit
    }));

    let loaded = store
        .claim_by_id(&claim.id)
        .expect("query succeeds")
        .expect("claim exists");
    assert_eq!(loaded.status, "doubted");
    assert_eq!(loaded.events.len(), 2);
    assert_eq!(loaded.events[1].kind, "trusted");
}

fn temp_project_root() -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("dont-store-test-{unique}"));
    fs::create_dir_all(&root).expect("temp root");
    root
}
