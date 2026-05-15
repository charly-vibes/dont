/// dont-4k5.1: Typed error hierarchy — callers can match on specific variants
/// without downcasting.
///
/// These tests verify the pass criteria: pub API returns typed Result<T, E>
/// where E is a concrete error enum/struct with matchable variants.
/// No anyhow::Error or Box<dyn Error> appears in any public signature.
use dont::config::ConfigValidationError;
use dont::model::{Status, TransitionError};
use dont::store::{Store, StoreError};

// ── StoreError variant matching ───────────────────────────────────────────────

/// Callers can match on StoreError::CurieConflict without any downcast.
/// If this compiles and passes, the public API is typed end-to-end.
#[test]
fn store_error_curie_conflict_is_matchable_without_downcast() {
    let root = tempfile::tempdir().expect("temp root");
    let store = Store::open_project(root.path()).expect("store opens");

    store
        .append_term("ex:foo", "first definition", None)
        .expect("first term appends");

    let err = store
        .append_term("ex:foo", "duplicate definition", None)
        .expect_err("duplicate curie must be rejected");

    // Direct variant match — no downcast, no .into(), no .source()
    match err {
        StoreError::CurieConflict { curie, existing_id } => {
            assert_eq!(curie, "ex:foo");
            assert!(existing_id.starts_with("term:"), "existing_id = {existing_id}");
        }
        other => panic!("expected CurieConflict, got {other:?}"),
    }
}

/// Callers can match on StoreError::SchemaMismatch without any downcast.
#[test]
fn store_error_schema_mismatch_is_matchable_without_downcast() {
    let root = tempfile::tempdir().expect("temp root");
    let store = Store::open_project(root.path()).expect("store opens");
    store
        .set_schema_version_for_test(999)
        .expect("seed incompatible version");
    drop(store);

    let err = Store::open_project(root.path()).expect_err("schema mismatch rejected");

    match err {
        StoreError::SchemaMismatch { found, expected } => {
            assert_eq!(found, 999);
            assert!(expected > 0, "expected schema version must be positive");
        }
        other => panic!("expected SchemaMismatch, got {other:?}"),
    }
}

// ── TransitionError field access without downcast ─────────────────────────────

/// Callers can inspect TransitionError fields directly — no downcasting needed.
#[test]
fn transition_error_fields_accessible_without_downcast() {
    // trust() on an already-doubted claim must return TransitionError
    let err: TransitionError = dont::model::trust(Status::Doubted).unwrap_err();

    // All fields are directly accessible — no Any, no downcast_ref
    assert_eq!(err.code, "invalid-transition");
    assert_eq!(err.from_status, Status::Doubted);
    assert_eq!(err.message, "cannot trust a Doubted entity");
    // entity_id is None when constructed without context
    assert!(err.entity_id.is_none());
}

// ── ConfigValidationError field access without downcast ───────────────────────

/// ConfigValidationError carries a message field directly accessible to callers.
/// Config::validate() returns Result<(), ConfigValidationError> — fully typed.
#[test]
fn config_validation_error_is_matchable_without_downcast() {
    use dont::config::Config;

    // Construct a Config that will fail validation: verify_evidence.default_timeout_s = 0.
    // Include the required [project] and [output] fields so validation reaches the
    // verify_evidence check rather than stopping at a missing-field error first.
    let toml_str = r#"
[project]
mode = "permissive"

[output]
default_format = "json"

[verify_evidence]
default_timeout_s = 0
"#;

    let config: Config = toml::from_str(toml_str).expect("TOML parses");
    let err: ConfigValidationError = config.validate().unwrap_err();

    // Direct field access — no downcast
    assert!(
        err.message.contains("default_timeout_s") || err.message.contains("timeout"),
        "validation error message must name the invalid field, got: {:?}",
        err.message
    );
}
