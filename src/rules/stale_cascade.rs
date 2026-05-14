use crate::store::{Store, StoreError, StoreStatus};

use super::RuleMatch;

pub const EXPLANATION: &str = include_str!("stale_cascade.md");

/// Fires when a claim depends on a term that is doubted or unverified (not yet verified).
///
/// "Stale" is derived on demand — this rule does not persist any status. Cycle-safe
/// because claims currently depend only on terms, not on other claims.
pub fn check(store: &Store) -> Result<Vec<RuleMatch>, StoreError> {
    let claims = store.list_claims()?;
    let mut matches = Vec::new();
    for claim in claims {
        // Spec: locked and ignored entities are exempt from derived stale output.
        if matches!(claim.status, StoreStatus::Ignored | StoreStatus::Locked) {
            continue;
        }
        for dep in &claim.depends_on {
            let term = if dep.starts_with("term:") {
                store.term_by_id(dep)?
            } else {
                store.term_by_curie(dep)?
            };
            if let Some(term) = term {
                if matches!(term.status, StoreStatus::Unverified | StoreStatus::Doubted) {
                    matches.push(RuleMatch {
                        entity_id: claim.id.clone(),
                        detail: format!(
                            "depends on '{}' with status '{}'",
                            term.curie,
                            term.status.as_str()
                        ),
                    });
                }
            }
        }
    }
    Ok(matches)
}

#[cfg(test)]
mod stale_cascade {
    use tempfile::TempDir;

    use super::*;
    use crate::store::{Store, StoreEvent, StoreEventKind, StoreStatus};

    fn make_store(dir: &TempDir) -> Store {
        Store::open_dont_dir(dir.path()).unwrap()
    }

    #[test]
    fn silent_when_dep_is_verified() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let term = store.append_term("ns:concept", "", None).unwrap();
        store
            .append_term_status_change(
                &term.id,
                StoreStatus::Unverified,
                StoreStatus::Verified,
                StoreEvent { kind: StoreEventKind::Flagged, note: None, evidence: vec![] },
            )
            .unwrap();
        store
            .append_claim("a claim", &["ns:concept".to_string()], None)
            .unwrap();
        assert!(check(&store).unwrap().is_empty());
    }

    #[test]
    fn fires_when_dep_is_unverified() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store.append_term("ns:concept", "", None).unwrap(); // stays unverified
        store
            .append_claim("a claim", &["ns:concept".to_string()], None)
            .unwrap();
        let matches = check(&store).unwrap();
        assert_eq!(matches.len(), 1);
        assert!(matches[0].detail.contains("unverified"));
    }

    #[test]
    fn fires_when_dep_is_doubted() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let term = store.append_term("ns:concept", "", None).unwrap();
        store
            .append_term_status_change(
                &term.id,
                StoreStatus::Unverified,
                StoreStatus::Doubted,
                StoreEvent { kind: StoreEventKind::Flagged, note: None, evidence: vec![] },
            )
            .unwrap();
        store
            .append_claim("a claim", &["ns:concept".to_string()], None)
            .unwrap();
        let matches = check(&store).unwrap();
        assert_eq!(matches.len(), 1);
        assert!(matches[0].detail.contains("doubted"));
    }

    #[test]
    fn silent_when_no_deps() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store.append_claim("a claim", &[], None).unwrap();
        assert!(check(&store).unwrap().is_empty());
    }

    #[test]
    fn silent_when_dep_not_found() {
        // Unresolved deps are handled by ungrounded/unresolved_terms, not stale_cascade.
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store
            .append_claim("a claim", &["ns:nonexistent".to_string()], None)
            .unwrap();
        assert!(check(&store).unwrap().is_empty());
    }

    // Spec: "locked and ignored entities are exempt from derived stale output"
    #[test]
    fn silent_for_locked_claim_with_doubted_dep() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let term = store.append_term("ns:concept", "", None).unwrap();
        store
            .append_term_status_change(
                &term.id,
                StoreStatus::Unverified,
                StoreStatus::Doubted,
                StoreEvent { kind: StoreEventKind::Flagged, note: None, evidence: vec![] },
            )
            .unwrap();
        let claim = store
            .append_claim("a locked claim", &["ns:concept".to_string()], None)
            .unwrap();
        // Transition claim to Locked (from Verified, as lock requires verified status)
        store
            .append_status_change(
                &claim.id,
                StoreStatus::Unverified,
                StoreStatus::Verified,
                StoreEvent { kind: StoreEventKind::Flagged, note: None, evidence: vec![] },
            )
            .unwrap();
        store
            .append_status_change(
                &claim.id,
                StoreStatus::Verified,
                StoreStatus::Locked,
                StoreEvent { kind: StoreEventKind::Locked, note: None, evidence: vec![] },
            )
            .unwrap();
        // Locked claims must be exempt from stale-cascade output.
        assert!(
            check(&store).unwrap().is_empty(),
            "locked claim should be exempt from stale-cascade"
        );
    }

    // Spec: "locked and ignored entities are exempt from derived stale output"
    #[test]
    fn silent_for_ignored_claim_with_doubted_dep() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let term = store.append_term("ns:concept", "", None).unwrap();
        store
            .append_term_status_change(
                &term.id,
                StoreStatus::Unverified,
                StoreStatus::Doubted,
                StoreEvent { kind: StoreEventKind::Flagged, note: None, evidence: vec![] },
            )
            .unwrap();
        let claim = store
            .append_claim("an ignored claim", &["ns:concept".to_string()], None)
            .unwrap();
        store
            .append_status_change(
                &claim.id,
                StoreStatus::Unverified,
                StoreStatus::Ignored,
                StoreEvent { kind: StoreEventKind::Ignored, note: None, evidence: vec![] },
            )
            .unwrap();
        // Ignored claims must be exempt from stale-cascade output.
        assert!(
            check(&store).unwrap().is_empty(),
            "ignored claim should be exempt from stale-cascade"
        );
    }
}
