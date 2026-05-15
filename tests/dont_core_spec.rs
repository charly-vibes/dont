// dont-lc2z: SPEC-ALIGN — dont-core spec verification
//
// Tests map to requirements in openspec/specs/dont-core/spec.md:
//
//   REQ-1  Epistemic forcing-function purpose
//   REQ-2  Core entity scope (claims and terms only)
//   REQ-3  Tool independence (no wai/beads runtime coupling)
//   REQ-4  Append-only event history (no destructive deletion)
//   REQ-5  Verification is justified status, not absolute truth
//   REQ-6  Core invariants constrain future capabilities

mod common;

use common::{dont, init_dir};
use dont::store::{Store, StoreEvent, StoreEventKind, Status};
use serde_json::Value;
use tempfile::TempDir;

// ── REQ-1: Epistemic forcing-function purpose ─────────────────────────────────

/// The CLI `--help` output must contain "epistemic" (case-insensitive) to confirm
/// the tool is framed as an epistemic forcing-function, not a generic task tracker.
#[test]
fn cli_about_text_contains_epistemic() {
    let out = dont()
        .args(["--help"])
        .output()
        .expect("dont --help runs");
    let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
    assert!(
        text.contains("epistemic"),
        "CLI --help must mention 'epistemic': {text}"
    );
}

/// The CLI `--help` output must contain "grounded" or "grounding" to confirm
/// the tool is framed around requiring grounding before verification.
#[test]
fn cli_about_text_references_grounding() {
    let out = dont()
        .args(["--help"])
        .output()
        .expect("dont --help runs");
    let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
    assert!(
        text.contains("ground"),
        "CLI --help must mention grounding: {text}"
    );
}

// ── REQ-2: Core entity scope (claims and terms only) ─────────────────────────

/// The store must recognise exactly two first-class entity kinds: `claim` and `term`.
/// All entity IDs returned by append operations must carry one of these prefixes.
#[test]
fn store_entity_ids_are_claims_or_terms() {
    let root = TempDir::new().unwrap();
    let store = Store::open_project(root.path()).expect("store opens");

    let claim = store
        .append_claim("only claims and terms are first-class", &[], None)
        .expect("claim appends");
    let term = store
        .append_term("ex:T001", "coined term for entity scope test", None)
        .expect("term appends");

    assert!(
        claim.id.starts_with("claim:"),
        "claim ID must start with 'claim:'; got {}",
        claim.id
    );
    assert!(
        term.id.starts_with("term:"),
        "term ID must start with 'term:'; got {}",
        term.id
    );
}

/// The store does not expose a way to create a third entity kind.
/// Listing all entities must produce only claim: and term: prefixed IDs.
#[test]
fn store_list_contains_only_claim_and_term_entities() {
    let root = TempDir::new().unwrap();
    let store = Store::open_project(root.path()).expect("store opens");

    store
        .append_claim("first-class claim", &[], None)
        .expect("claim appends");
    store
        .append_term("ex:T001", "first-class term", None)
        .expect("term appends");

    let claims = store.list_claims().expect("list_claims succeeds");
    let terms = store.list_terms().expect("list_terms succeeds");

    for c in &claims {
        assert!(
            c.id.starts_with("claim:"),
            "every entry in list_claims must have a 'claim:' prefix; got {}",
            c.id
        );
    }
    for t in &terms {
        assert!(
            t.id.starts_with("term:"),
            "every entry in list_terms must have a 'term:' prefix; got {}",
            t.id
        );
    }
}

/// The CLI `conclude` command must create a `claim` entity kind.
#[test]
fn conclude_creates_entity_of_kind_claim() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["conclude", "only claims and terms exist", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["data"]["entity_kind"], "claim",
        "conclude must produce entity_kind='claim'"
    );
}

/// The CLI `define` command must create a `term` entity kind.
#[test]
fn define_creates_entity_of_kind_term() {
    let dir = TempDir::new().unwrap();
    init_dir(&dir);

    let out = dont()
        .args(["define", "ex:T001", "--doc", "a coined term", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["data"]["entity_kind"], "term",
        "define must produce entity_kind='term'"
    );
}

// ── REQ-3: Tool independence ──────────────────────────────────────────────────

/// dont operates without any wai or beads configuration.
/// A freshly initialized project (no DONT_WAI_DIR, no DONT_BEADS_DIR)
/// must accept and store a claim without error.
#[test]
fn dont_operates_without_wai_or_beads_config() {
    let dir = TempDir::new().unwrap();

    // Initialise with only DONT_DIR — no wai/beads env vars
    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", dir.path())
        // Explicitly remove any accidental wai/beads env that might be present
        .env_remove("WAI_DIR")
        .env_remove("BEADS_DIR")
        .env_remove("BD_DIR")
        .assert()
        .success();

    let out = dont()
        .args(["conclude", "tool independence holds", "--json"])
        .env("DONT_DIR", dir.path())
        .env_remove("WAI_DIR")
        .env_remove("BEADS_DIR")
        .env_remove("BD_DIR")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["ok"], true, "claim must be stored without companion tools");
}

// ── REQ-4: Append-only event history ─────────────────────────────────────────

/// After a status change, the original creation event must still appear in the
/// event history. Events accumulate; nothing is deleted.
#[test]
fn status_change_preserves_creation_event_in_history() {
    let root = TempDir::new().unwrap();
    let store = Store::open_project(root.path()).expect("store opens");

    let claim = store
        .append_claim("history is append-only", &[], None)
        .expect("claim appends");

    store
        .append_status_change(
            &claim.id,
            Status::Unverified,
            Status::Doubted,
            StoreEvent {
                kind: StoreEventKind::Trusted,
                note: Some("raise doubt".to_string()),
                evidence: vec![],
            },
        )
        .expect("status change appends");

    let loaded = store
        .claim_by_id(&claim.id)
        .expect("query succeeds")
        .expect("claim exists");

    // Must have original Concluded event PLUS the Trusted event
    assert_eq!(
        loaded.events.len(),
        2,
        "history must accumulate: got {:?}",
        loaded.events.iter().map(|e| e.kind).collect::<Vec<_>>()
    );
    assert_eq!(
        loaded.events[0].kind,
        StoreEventKind::Concluded,
        "first event must be the original Concluded"
    );
    assert_eq!(
        loaded.events[1].kind,
        StoreEventKind::Trusted,
        "second event must be the Trusted transition"
    );
}

/// Retraction events add a new event row, they do not delete the prior status event.
/// The datom log must show both the retraction of the old status and the assertion
/// of the new status at the same tx, with the original assertion still present.
#[test]
fn retraction_event_is_recorded_alongside_original_assertion() {
    let root = TempDir::new().unwrap();
    let store = Store::open_project(root.path()).expect("store opens");

    let claim = store
        .append_claim("retraction extends history, never erases it", &[], None)
        .expect("claim appends");

    let transition = store
        .append_status_change(
            &claim.id,
            Status::Unverified,
            Status::Doubted,
            StoreEvent {
                kind: StoreEventKind::Trusted,
                note: None,
                evidence: vec![],
            },
        )
        .expect("status change appends");

    let datoms = store
        .datoms_for_entity(&claim.id)
        .expect("datoms readable");

    // The original assert=true for 'unverified' at claim.tx must still exist
    let original_assert = datoms.iter().any(|d| {
        d.attribute == "status"
            && d.value == serde_json::json!("unverified")
            && d.assert_bit
            && d.tx == claim.tx
    });
    assert!(
        original_assert,
        "original unverified assertion datom must remain in log"
    );

    // The transition tx must carry a retract for 'unverified'
    let retract = datoms.iter().any(|d| {
        d.attribute == "status"
            && d.value == serde_json::json!("unverified")
            && !d.assert_bit
            && d.tx == transition.tx
    });
    assert!(
        retract,
        "retraction datom for old status must be recorded at transition tx"
    );

    // And an assert for 'doubted' at transition tx
    let new_assert = datoms.iter().any(|d| {
        d.attribute == "status"
            && d.value == serde_json::json!("doubted")
            && d.assert_bit
            && d.tx == transition.tx
    });
    assert!(
        new_assert,
        "new status assertion datom must be recorded at transition tx"
    );
}

// ── REQ-5: Verification is justified status, not absolute truth ───────────────

/// A newly created claim must have `unverified` status — it cannot start verified.
/// This enforces that verification requires an explicit justification step.
#[test]
fn newly_created_claim_starts_as_unverified_not_verified() {
    let root = TempDir::new().unwrap();
    let store = Store::open_project(root.path()).expect("store opens");

    let claim = store
        .append_claim("all conclusions start unverified", &[], None)
        .expect("claim appends");

    let loaded = store
        .claim_by_id(&claim.id)
        .expect("query succeeds")
        .expect("claim exists");

    assert_eq!(
        loaded.status,
        Status::Unverified,
        "new claim must start as unverified — verification requires explicit justification"
    );
}

/// A newly defined term must have `unverified` status.
#[test]
fn newly_defined_term_starts_as_unverified_not_verified() {
    let root = TempDir::new().unwrap();
    let store = Store::open_project(root.path()).expect("store opens");

    let term = store
        .append_term("ex:T001", "terms also require explicit verification", None)
        .expect("term appends");

    let loaded = store
        .term_by_id(&term.id)
        .expect("query succeeds")
        .expect("term exists");

    assert_eq!(
        loaded.status,
        Status::Unverified,
        "new term must start as unverified — verification requires justification"
    );
}

/// The model layer must not allow direct status-setting to `verified` from `verified`.
/// `verified` can only be reached via an explicit flag transition from an active status.
#[test]
fn verified_status_is_reached_only_via_flag_transition() {
    use dont::model::{Status, flag};

    // flag(Unverified) → Verified is the only direct path to verified
    assert_eq!(
        flag(Status::Unverified).unwrap(),
        Status::Verified,
        "flag from unverified must produce verified"
    );
    // flag(Doubted) → Verified is also valid (re-evaluation after scrutiny)
    assert_eq!(
        flag(Status::Doubted).unwrap(),
        Status::Verified,
        "flag from doubted must produce verified"
    );
    // flag(Verified) is refused — verified-to-verified is not a justification step
    let err = flag(Status::Verified).unwrap_err();
    assert_eq!(
        err.code, "invalid-transition",
        "flag from verified must be refused"
    );
}

// ── REQ-6: Core invariants ────────────────────────────────────────────────────

/// Standalone operation: dont init and conclude work in a temp directory
/// that has no `.wai`, `.beads`, or other tool state present.
#[test]
fn standalone_operation_without_companion_tool_directories() {
    let dir = TempDir::new().unwrap();

    // Confirm no wai/beads directories exist
    assert!(
        !dir.path().join(".wai").exists(),
        "test env must not have .wai"
    );
    assert!(
        !dir.path().join(".beads").exists(),
        "test env must not have .beads"
    );

    // Init and conclude must succeed in this bare environment
    dont()
        .args(["init", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success();

    let out = dont()
        .args(["conclude", "invariants hold standalone", "--json"])
        .env("DONT_DIR", dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["ok"], true,
        "standalone dont conclude must succeed: {v}"
    );
}

/// Append-only invariant: tx values must be monotonically increasing across
/// multiple operations, confirming the log-append semantics hold.
#[test]
fn tx_values_are_strictly_monotonic_across_operations() {
    let root = TempDir::new().unwrap();
    let store = Store::open_project(root.path()).expect("store opens");

    let c1 = store.append_claim("first claim", &[], None).expect("c1");
    let c2 = store.append_claim("second claim", &[], None).expect("c2");
    let t1 = store
        .append_term("ex:T001", "first term", None)
        .expect("t1");

    assert!(c1.tx < c2.tx, "tx must increase: c1.tx={} c2.tx={}", c1.tx, c2.tx);
    assert!(c2.tx < t1.tx, "tx must increase: c2.tx={} t1.tx={}", c2.tx, t1.tx);
}

/// Explicit epistemic status invariant: every entity returned from the store
/// must carry an explicit status string (never empty or null).
#[test]
fn every_entity_has_explicit_epistemic_status() {
    let root = TempDir::new().unwrap();
    let store = Store::open_project(root.path()).expect("store opens");

    store.append_claim("claim must have status", &[], None).expect("c1");
    store
        .append_term("ex:T001", "term must have status", None)
        .expect("t1");

    let claims = store.list_claims().expect("list_claims");
    let terms = store.list_terms().expect("list_terms");

    for c in &claims {
        let status_str = c.status.as_str();
        assert!(
            !status_str.is_empty(),
            "claim {} must have non-empty status",
            c.id
        );
    }
    for t in &terms {
        let status_str = t.status.as_str();
        assert!(
            !status_str.is_empty(),
            "term {} must have non-empty status",
            t.id
        );
    }
}
