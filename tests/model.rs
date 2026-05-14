use dont::model::{Status, TransitionError, flag, ignore, lock, reopen, trust, undoubt};

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
fn flag_unverified_produces_verified() {
    assert_eq!(flag(Status::Unverified).unwrap(), Status::Verified);
}

#[test]
fn flag_doubted_produces_verified() {
    assert_eq!(flag(Status::Doubted).unwrap(), Status::Verified);
}

#[test]
fn undoubt_doubted_produces_unverified() {
    assert_eq!(undoubt(Status::Doubted).unwrap(), Status::Unverified);
}

#[test]
fn lock_verified_produces_locked() {
    assert_eq!(lock(Status::Verified).unwrap(), Status::Locked);
}

// --- Invalid transitions (typed refusal) ---

#[test]
fn trust_doubted_is_refused() {
    let err = trust(Status::Doubted).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(!err.message.is_empty());
}

#[test]
fn flag_verified_is_refused() {
    // Already-verified flag is evidence append, not a status transition.
    // The model function must return a typed refusal.
    let err = flag(Status::Verified).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(!err.message.is_empty());
}

#[test]
fn undoubt_unverified_is_refused() {
    let err = undoubt(Status::Unverified).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(!err.message.is_empty());
}

#[test]
fn undoubt_verified_is_refused() {
    let err = undoubt(Status::Verified).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(!err.message.is_empty());
}

#[test]
fn lock_unverified_is_refused() {
    let err = lock(Status::Unverified).unwrap_err();
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

// --- Complete invalid-transition matrix coverage ---
// Every (state, operation) pair that is invalid must return a typed Err
// with a non-empty message identifying the state and operation.

// trust: only Unverified and Verified are valid sources
#[test]
fn trust_ignored_is_refused() {
    let err = trust(Status::Ignored).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(err.message.contains("Ignored") || err.message.contains("ignored"),
        "message should identify the state: {}", err.message);
    assert_eq!(err.from_status, Status::Ignored);
}

#[test]
fn trust_locked_is_refused() {
    let err = trust(Status::Locked).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(err.message.contains("Locked") || err.message.contains("locked"),
        "message should identify the state: {}", err.message);
    assert_eq!(err.from_status, Status::Locked);
}

// flag: only Unverified and Doubted are valid sources
#[test]
fn flag_ignored_is_refused() {
    let err = flag(Status::Ignored).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(err.message.contains("Ignored") || err.message.contains("ignored"),
        "message should identify the state: {}", err.message);
    assert_eq!(err.from_status, Status::Ignored);
}

#[test]
fn flag_locked_is_refused() {
    let err = flag(Status::Locked).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(err.message.contains("Locked") || err.message.contains("locked"),
        "message should identify the state: {}", err.message);
    assert_eq!(err.from_status, Status::Locked);
}

// ignore: only Unverified, Verified, Doubted are valid sources
#[test]
fn ignore_ignored_is_refused() {
    let err = ignore(Status::Ignored).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(!err.message.is_empty(), "error message must not be empty");
    assert_eq!(err.from_status, Status::Ignored);
}

#[test]
fn ignore_locked_is_refused() {
    let err = ignore(Status::Locked).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(err.message.contains("locked") || err.message.contains("Locked"),
        "message should identify the state: {}", err.message);
    assert_eq!(err.from_status, Status::Locked);
}

// reopen: only Ignored is a valid source
#[test]
fn reopen_unverified_is_refused() {
    let err = reopen(Status::Unverified).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(err.message.contains("Unverified") || err.message.contains("unverified"),
        "message should identify the state: {}", err.message);
    assert_eq!(err.from_status, Status::Unverified);
}

#[test]
fn reopen_verified_is_refused() {
    let err = reopen(Status::Verified).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(err.message.contains("Verified") || err.message.contains("verified"),
        "message should identify the state: {}", err.message);
    assert_eq!(err.from_status, Status::Verified);
}

#[test]
fn reopen_doubted_is_refused() {
    let err = reopen(Status::Doubted).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(err.message.contains("Doubted") || err.message.contains("doubted"),
        "message should identify the state: {}", err.message);
    assert_eq!(err.from_status, Status::Doubted);
}

#[test]
fn reopen_locked_is_refused() {
    let err = reopen(Status::Locked).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(err.message.contains("Locked") || err.message.contains("locked"),
        "message should identify the state: {}", err.message);
    assert_eq!(err.from_status, Status::Locked);
}

// undoubt: only Doubted is a valid source
#[test]
fn undoubt_ignored_is_refused() {
    let err = undoubt(Status::Ignored).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(err.message.contains("Ignored") || err.message.contains("ignored"),
        "message should identify the state: {}", err.message);
    assert_eq!(err.from_status, Status::Ignored);
}

#[test]
fn undoubt_locked_is_refused() {
    let err = undoubt(Status::Locked).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(err.message.contains("Locked") || err.message.contains("locked"),
        "message should identify the state: {}", err.message);
    assert_eq!(err.from_status, Status::Locked);
}

// lock: only Verified is a valid source
#[test]
fn lock_doubted_is_refused() {
    let err = lock(Status::Doubted).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(err.message.contains("Doubted") || err.message.contains("doubted"),
        "message should identify the state: {}", err.message);
    assert_eq!(err.from_status, Status::Doubted);
}

#[test]
fn lock_ignored_is_refused() {
    let err = lock(Status::Ignored).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(err.message.contains("Ignored") || err.message.contains("ignored"),
        "message should identify the state: {}", err.message);
    assert_eq!(err.from_status, Status::Ignored);
}

#[test]
fn lock_locked_is_refused() {
    let err = lock(Status::Locked).unwrap_err();
    assert_eq!(err.code, "invalid-transition");
    assert!(err.message.contains("Locked") || err.message.contains("locked"),
        "message should identify the state: {}", err.message);
    assert_eq!(err.from_status, Status::Locked);
}

// --- Valid transitions completeness ---
// Confirm that ignore and reopen valid paths also work
#[test]
fn ignore_unverified_produces_ignored() {
    assert_eq!(ignore(Status::Unverified).unwrap(), Status::Ignored);
}

#[test]
fn ignore_verified_produces_ignored() {
    assert_eq!(ignore(Status::Verified).unwrap(), Status::Ignored);
}

#[test]
fn ignore_doubted_produces_ignored() {
    assert_eq!(ignore(Status::Doubted).unwrap(), Status::Ignored);
}

#[test]
fn reopen_ignored_produces_unverified() {
    assert_eq!(reopen(Status::Ignored).unwrap(), Status::Unverified);
}
