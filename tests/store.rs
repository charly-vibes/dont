use dont::store::{Status, Store, StoreError, StoreEvent, StoreEventKind};

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

    assert!(
        matches!(error, StoreError::SchemaMismatch { found: 999, .. }),
        "expected SchemaMismatch with found=999, got: {error}"
    );
}

#[test]
fn append_event_persists_claim_datoms_with_monotonic_transactions() {
    let root = tempfile::tempdir().expect("temp root");
    let store = Store::open_project(root.path()).expect("store opens");

    let first = store
        .append_claim("unsupported assertions need grounding", &[], None)
        .expect("first claim appends");
    let second = store
        .append_claim("sources should be explicit", &[], None)
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
    assert_eq!(loaded.status, Status::Unverified);
    assert_eq!(loaded.events.len(), 1);
    assert_eq!(loaded.events[0].kind, StoreEventKind::Concluded);
    assert_eq!(loaded.events[0].tx, first.tx);
}

#[test]
fn tx_ids_are_unique_across_concurrent_store_instances() {
    use std::sync::Arc;
    use std::thread;

    let root = tempfile::tempdir().expect("temp root");

    // Open multiple Store instances sequentially (simulating separate processes that have
    // already initialized), then run their appends concurrently.
    let stores: Vec<Arc<Store>> = (0..8)
        .map(|_| Arc::new(Store::open_project(root.path()).expect("store opens")))
        .collect();

    let handles: Vec<_> = stores
        .into_iter()
        .enumerate()
        .map(|(i, store)| {
            thread::spawn(move || {
                store
                    .append_claim(&format!("claim {i}"), &[], None)
                    .expect("append succeeds")
                    .tx
            })
        })
        .collect();

    let mut tx_ids: Vec<i64> = handles
        .into_iter()
        .map(|h| h.join().expect("thread succeeded"))
        .collect();

    tx_ids.sort_unstable();
    tx_ids.dedup();
    assert_eq!(tx_ids.len(), 8, "all tx IDs must be unique; got {tx_ids:?}");
}

#[test]
fn claims_persist_after_reopening_the_store() {
    let root = tempfile::tempdir().expect("temp root");
    let claim_id = {
        let store = Store::open_project(root.path()).expect("store opens");
        store
            .append_claim("memory survives process boundaries", &[], None)
            .expect("claim appends")
            .id
    };

    let reopened = Store::open_project(root.path()).expect("store reopens");
    let loaded = reopened
        .claim_by_id(&claim_id)
        .expect("query succeeds")
        .expect("claim exists after reopen");

    assert_eq!(loaded.statement, "memory survives process boundaries");
    assert_eq!(loaded.status, Status::Unverified);
}

#[test]
fn status_changes_are_stored_as_retraction_and_assertion_datoms() {
    let root = tempfile::tempdir().expect("temp root");
    let store = Store::open_project(root.path()).expect("store opens");

    let claim = store
        .append_claim("truth needs pressure", &[], None)
        .expect("claim");
    let event = StoreEvent {
        kind: StoreEventKind::Trusted,
        note: Some("source is ambiguous".to_string()),
        evidence: vec![],
    };

    let transition = store
        .append_status_change(&claim.id, Status::Unverified, Status::Doubted, event)
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
    assert_eq!(loaded.status, Status::Doubted);
    assert_eq!(loaded.events.len(), 2);
    assert_eq!(loaded.events[1].kind, StoreEventKind::Trusted);
}

// ── Ordering invariant tests (dont-4k5.2) ────────────────────────────────────

/// After multiple sequential status changes, claim_by_id must return events
/// sorted by tx (ascending insertion order), not by ID or arbitrary DB order.
#[test]
fn claim_events_are_replayed_in_tx_insertion_order() {
    use dont::store::StoreEventKind;

    let root = tempfile::tempdir().expect("temp root");
    let store = Store::open_project(root.path()).expect("store opens");

    let claim = store
        .append_claim("ordering must be deterministic", &[], None)
        .expect("claim appends");

    let flag_result = store
        .append_status_change(
            &claim.id,
            Status::Unverified,
            Status::Doubted,
            StoreEvent {
                kind: StoreEventKind::Flagged,
                note: Some("step 1: flag".to_string()),
                evidence: vec![],
            },
        )
        .expect("flag appends");

    let reopen_result = store
        .append_status_change(
            &claim.id,
            Status::Doubted,
            Status::Unverified,
            StoreEvent {
                kind: StoreEventKind::Reopened,
                note: Some("step 2: reopen".to_string()),
                evidence: vec![],
            },
        )
        .expect("reopen appends");

    // tx values must be strictly increasing
    assert!(claim.tx < flag_result.tx, "flag tx must follow initial tx");
    assert!(
        flag_result.tx < reopen_result.tx,
        "reopen tx must follow flag tx"
    );

    let loaded = store
        .claim_by_id(&claim.id)
        .expect("query succeeds")
        .expect("claim exists");

    assert_eq!(loaded.events.len(), 3, "must have 3 events");

    // Events must be in tx order — Concluded → Flagged → Reopened
    assert_eq!(
        loaded.events[0].kind,
        StoreEventKind::Concluded,
        "first event must be Concluded"
    );
    assert_eq!(
        loaded.events[1].kind,
        StoreEventKind::Flagged,
        "second event must be Flagged"
    );
    assert_eq!(
        loaded.events[2].kind,
        StoreEventKind::Reopened,
        "third event must be Reopened"
    );

    // tx values on each EventRecord must match the AppendResult tx values
    assert_eq!(loaded.events[0].tx, claim.tx);
    assert_eq!(loaded.events[1].tx, flag_result.tx);
    assert_eq!(loaded.events[2].tx, reopen_result.tx);

    // Confirm events slice is sorted ascending by tx
    let txs: Vec<i64> = loaded.events.iter().map(|e| e.tx).collect();
    let mut sorted = txs.clone();
    sorted.sort_unstable();
    assert_eq!(txs, sorted, "event tx values must be in ascending order");
}

/// list_claims must return each claim's events sorted by tx — same guarantee
/// as claim_by_id — confirming the bulk-list path also preserves order.
#[test]
fn list_claims_events_are_in_tx_order() {
    use dont::store::StoreEventKind;

    let root = tempfile::tempdir().expect("temp root");
    let store = Store::open_project(root.path()).expect("store opens");

    let c1 = store
        .append_claim("first ordering claim", &[], None)
        .expect("first claim");

    // Two status changes on the same claim
    let flag = store
        .append_status_change(
            &c1.id,
            Status::Unverified,
            Status::Doubted,
            StoreEvent {
                kind: StoreEventKind::Flagged,
                note: None,
                evidence: vec![],
            },
        )
        .expect("flag");
    let reopen = store
        .append_status_change(
            &c1.id,
            Status::Doubted,
            Status::Unverified,
            StoreEvent {
                kind: StoreEventKind::Reopened,
                note: None,
                evidence: vec![],
            },
        )
        .expect("reopen");

    let claims = store.list_claims().expect("list_claims succeeds");
    let found = claims
        .into_iter()
        .find(|c| c.id == c1.id)
        .expect("claim appears in list");

    assert_eq!(found.events.len(), 3);
    let txs: Vec<i64> = found.events.iter().map(|e| e.tx).collect();
    let mut sorted = txs.clone();
    sorted.sort_unstable();
    assert_eq!(
        txs, sorted,
        "list_claims event order must match tx insertion order"
    );

    // Exact sequence must be Concluded → Flagged → Reopened
    assert_eq!(found.events[0].tx, c1.tx);
    assert_eq!(found.events[1].tx, flag.tx);
    assert_eq!(found.events[2].tx, reopen.tx);
}

/// list_terms must return each term's events sorted by tx.
#[test]
fn list_terms_events_are_in_tx_order() {
    use dont::store::StoreEventKind;

    let root = tempfile::tempdir().expect("temp root");
    let store = Store::open_project(root.path()).expect("store opens");

    let t1 = store
        .append_term("ex:term1", "a well-ordered term", None)
        .expect("term appends");

    let trust = store
        .append_term_status_change(
            &t1.id,
            Status::Unverified,
            Status::Verified,
            StoreEvent {
                kind: StoreEventKind::Trusted,
                note: Some("source confirmed".to_string()),
                evidence: vec![],
            },
        )
        .expect("trust appends");

    let doubt = store
        .append_term_status_change(
            &t1.id,
            Status::Verified,
            Status::Doubted,
            StoreEvent {
                kind: StoreEventKind::Flagged,
                note: Some("doubt raised".to_string()),
                evidence: vec![],
            },
        )
        .expect("doubt appends");

    assert!(t1.tx < trust.tx, "trust tx must follow initial tx");
    assert!(trust.tx < doubt.tx, "doubt tx must follow trust tx");

    let terms = store.list_terms().expect("list_terms succeeds");
    let found = terms
        .into_iter()
        .find(|t| t.id == t1.id)
        .expect("term appears in list");

    assert_eq!(
        found.events.len(),
        3,
        "must have 3 events: Defined + Trusted + Flagged"
    );

    let txs: Vec<i64> = found.events.iter().map(|e| e.tx).collect();
    let mut sorted_txs = txs.clone();
    sorted_txs.sort_unstable();
    assert_eq!(
        txs, sorted_txs,
        "list_terms event order must match tx insertion order"
    );

    assert_eq!(found.events[0].kind, StoreEventKind::Defined);
    assert_eq!(found.events[1].kind, StoreEventKind::Trusted);
    assert_eq!(found.events[2].kind, StoreEventKind::Flagged);
}

/// Replay determinism: re-opening the store after multiple appends must
/// produce events in the same tx order as when the store was first written.
#[test]
fn event_order_is_stable_across_store_reopen() {
    use dont::store::StoreEventKind;

    let root = tempfile::tempdir().expect("temp root");
    let (claim_id, expected_kinds) = {
        let store = Store::open_project(root.path()).expect("store opens");
        let claim = store
            .append_claim("replay must be deterministic", &[], None)
            .expect("claim");
        store
            .append_status_change(
                &claim.id,
                Status::Unverified,
                Status::Doubted,
                StoreEvent {
                    kind: StoreEventKind::Flagged,
                    note: None,
                    evidence: vec![],
                },
            )
            .expect("flag");
        store
            .append_status_change(
                &claim.id,
                Status::Doubted,
                Status::Unverified,
                StoreEvent {
                    kind: StoreEventKind::Reopened,
                    note: None,
                    evidence: vec![],
                },
            )
            .expect("reopen");
        (
            claim.id,
            vec![
                StoreEventKind::Concluded,
                StoreEventKind::Flagged,
                StoreEventKind::Reopened,
            ],
        )
    };

    // Reopen and verify replay order is identical
    let store2 = Store::open_project(root.path()).expect("store reopens");
    let loaded = store2
        .claim_by_id(&claim_id)
        .expect("query succeeds")
        .expect("claim exists after reopen");

    let actual_kinds: Vec<StoreEventKind> = loaded.events.iter().map(|e| e.kind).collect();
    assert_eq!(
        actual_kinds, expected_kinds,
        "event order after reopen must match original insertion order"
    );

    // Also confirm tx is still monotonically ascending
    let txs: Vec<i64> = loaded.events.iter().map(|e| e.tx).collect();
    let mut sorted = txs.clone();
    sorted.sort_unstable();
    assert_eq!(txs, sorted, "tx values must be ascending after reopen");
}

// ── Corrupt store file tests (dont-bt3) ──────────────────────────────────────

/// A corrupt tx.seq (non-numeric content) must produce an error whose message
/// names the tx.seq file path so the user knows which file to delete or repair.
#[test]
fn corrupt_tx_seq_error_names_the_file_path() {
    let root = tempfile::tempdir().expect("temp root");

    // Write one claim so tx.seq is created.
    let store = Store::open_project(root.path()).expect("store opens");
    store
        .append_claim("establish tx.seq before corruption", &[], None)
        .expect("claim appends");
    drop(store);

    // Overwrite tx.seq with garbage.
    let seq_path = root.path().join(".dont/tx.seq");
    std::fs::write(&seq_path, b"not-a-number\n").expect("wrote garbage tx.seq");

    // Reopen and try an operation that reads tx.seq.
    let store2 = Store::open_project(root.path()).expect("store reopens after corruption");
    let err = store2
        .append_claim("this should fail on next_tx", &[], None)
        .expect_err("should fail with corrupt tx.seq");

    let msg = err.to_string();
    assert!(
        msg.contains("tx.seq"),
        "error message must name 'tx.seq', got: {msg}"
    );
    // The message should include the absolute path so the user can locate the file.
    assert!(
        msg.contains(seq_path.to_str().unwrap()),
        "error message must contain the absolute tx.seq path, got: {msg}"
    );
}

/// A corrupt db.cozo (truncated/invalid SQLite content) must produce an error
/// whose message names the db.cozo file path so the user knows which file is corrupt.
#[test]
fn corrupt_db_file_error_names_the_file_path() {
    let root = tempfile::tempdir().expect("temp root");

    // Initialize a valid store first (creates the SQLite file).
    {
        let _store = Store::open_project(root.path()).expect("store opens");
    }

    // Corrupt the SQLite file by overwriting it with garbage.
    let db_path = root.path().join(".dont/db.cozo");
    std::fs::write(&db_path, b"not a valid sqlite database\x00\x00\xff\xfe")
        .expect("wrote garbage db");

    // Attempting to reopen must return an error that names the db path.
    let err = Store::open_project(root.path()).expect_err("corrupt db must fail to open");

    let msg = err.to_string();
    assert!(
        msg.contains("db.cozo"),
        "error message must name 'db.cozo', got: {msg}"
    );
    assert!(
        msg.contains(db_path.to_str().unwrap()),
        "error message must contain the absolute db.cozo path, got: {msg}"
    );
}
