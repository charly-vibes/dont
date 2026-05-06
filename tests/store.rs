use dont::store::{Store, StoreEvent, StoreEventKind, StoreStatus};

#[test]
fn opens_project_store_at_canonical_path_and_records_schema_metadata() {
    let root = tempfile::tempdir().expect("temp root");

    let store = Store::open_project(root.path()).expect("store opens");

    assert!(root.path().join(".dont/db.cozo").exists());
    assert_eq!(store.schema_version().expect("schema version"), 1);
}

#[test]
fn rejects_incompatible_schema_versions_instead_of_overwriting_them() {
    let root = tempfile::tempdir().expect("temp root");
    let store = Store::open_project(root.path()).expect("store opens");
    store
        .set_schema_version_for_test(999)
        .expect("seed incompatible schema");
    drop(store);

    let error = Store::open_project(root.path()).expect_err("schema mismatch rejected");

    assert!(error.to_string().contains("unsupported schema_version 999"));
}

#[test]
fn append_event_persists_claim_datoms_with_monotonic_transactions() {
    let root = tempfile::tempdir().expect("temp root");
    let store = Store::open_project(root.path()).expect("store opens");

    let first = store
        .append_claim("unsupported assertions need grounding", &[])
        .expect("first claim appends");
    let second = store
        .append_claim("sources should be explicit", &[])
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
    assert_eq!(loaded.status, StoreStatus::Unverified);
    assert_eq!(loaded.events.len(), 1);
    assert_eq!(loaded.events[0].kind, StoreEventKind::Concluded);
    assert_eq!(loaded.events[0].tx, first.tx);
}

#[test]
fn claims_persist_after_reopening_the_store() {
    let root = tempfile::tempdir().expect("temp root");
    let claim_id = {
        let store = Store::open_project(root.path()).expect("store opens");
        store
            .append_claim("memory survives process boundaries", &[])
            .expect("claim appends")
            .id
    };

    let reopened = Store::open_project(root.path()).expect("store reopens");
    let loaded = reopened
        .claim_by_id(&claim_id)
        .expect("query succeeds")
        .expect("claim exists after reopen");

    assert_eq!(loaded.statement, "memory survives process boundaries");
    assert_eq!(loaded.status, StoreStatus::Unverified);
}

#[test]
fn status_changes_are_stored_as_retraction_and_assertion_datoms() {
    let root = tempfile::tempdir().expect("temp root");
    let store = Store::open_project(root.path()).expect("store opens");

    let claim = store.append_claim("truth needs pressure", &[]).expect("claim");
    let event = StoreEvent {
        kind: StoreEventKind::Trusted,
        note: Some("source is ambiguous".to_string()),
        evidence: vec![],
    };

    let transition = store
        .append_status_change(
            &claim.id,
            StoreStatus::Unverified,
            StoreStatus::Doubted,
            event,
        )
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
    assert_eq!(loaded.status, StoreStatus::Doubted);
    assert_eq!(loaded.events.len(), 2);
    assert_eq!(loaded.events[1].kind, StoreEventKind::Trusted);
}
