use crate::store::{Store, StoreError, StoreStatus};

use super::RuleMatch;

/// Fires when a claim depends on a term that is doubted or unverified (not yet verified).
///
/// "Stale" is derived on demand — this rule does not persist any status. Cycle-safe
/// because claims currently depend only on terms, not on other claims.
pub fn check(store: &Store) -> Result<Vec<RuleMatch>, StoreError> {
    let claims = store.list_claims()?;
    let mut matches = Vec::new();
    for claim in claims {
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
            .append_claim("a claim", &["ns:concept".to_string()])
            .unwrap();
        assert!(check(&store).unwrap().is_empty());
    }

    #[test]
    fn fires_when_dep_is_unverified() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store.append_term("ns:concept", "", None).unwrap(); // stays unverified
        store
            .append_claim("a claim", &["ns:concept".to_string()])
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
            .append_claim("a claim", &["ns:concept".to_string()])
            .unwrap();
        let matches = check(&store).unwrap();
        assert_eq!(matches.len(), 1);
        assert!(matches[0].detail.contains("doubted"));
    }

    #[test]
    fn silent_when_no_deps() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store.append_claim("a claim", &[]).unwrap();
        assert!(check(&store).unwrap().is_empty());
    }

    #[test]
    fn silent_when_dep_not_found() {
        // Unresolved deps are handled by ungrounded/unresolved_terms, not stale_cascade.
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store
            .append_claim("a claim", &["ns:nonexistent".to_string()])
            .unwrap();
        assert!(check(&store).unwrap().is_empty());
    }
}
