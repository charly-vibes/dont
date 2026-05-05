use dont::model::{Status, TransitionError, trust, dismiss};

// --- Valid transitions ---

#[test]
fn trust_unverified_produces_doubted() {
    assert_eq!(trust(Status::Unverified).unwrap(), Status::Doubted);
}

#[test]
fn trust_verified_produces_doubted() {
    assert_eq!(trust(Status::Verified).unwrap(), Status::Doubted);
}

#[test]
fn dismiss_unverified_produces_verified() {
    assert_eq!(dismiss(Status::Unverified).unwrap(), Status::Verified);
}

#[test]
fn dismiss_doubted_produces_verified() {
    assert_eq!(dismiss(Status::Doubted).unwrap(), Status::Verified);
}

// --- Invalid transitions (typed refusal) ---

#[test]
fn trust_doubted_is_refused() {
    let err = trust(Status::Doubted).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(!err.message.is_empty());
}

#[test]
fn dismiss_verified_is_refused() {
    // Already-verified dismiss is evidence append, not a status transition (Phase 8).
    // For now it must return a typed refusal.
    let err = dismiss(Status::Verified).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(!err.message.is_empty());
}

// --- Status serializes to lowercase kebab ---

#[test]
fn status_serializes_to_lowercase() {
    assert_eq!(
        serde_json::to_string(&Status::Unverified).unwrap(),
        "\"unverified\""
    );
    assert_eq!(
        serde_json::to_string(&Status::Verified).unwrap(),
        "\"verified\""
    );
    assert_eq!(
        serde_json::to_string(&Status::Doubted).unwrap(),
        "\"doubted\""
    );
}

#[test]
fn status_deserializes_from_lowercase() {
    let s: Status = serde_json::from_str("\"doubted\"").unwrap();
    assert_eq!(s, Status::Doubted);
}

// --- TransitionError carries entity ID context when provided ---

#[test]
fn transition_error_has_entity_id_field() {
    let err = TransitionError {
        code: "invalid-transition".to_string(),
        message: "cannot trust a doubted entity".to_string(),
        from_status: Status::Doubted,
        entity_id: Some("claim:01JTEST".to_string()),
    };
    assert_eq!(err.entity_id.as_deref(), Some("claim:01JTEST"));
}

#[test]
fn transition_error_without_entity_id() {
    let err = trust(Status::Doubted).unwrap_err();
    // entity_id is None when constructed without context
    assert!(err.entity_id.is_none());
}
