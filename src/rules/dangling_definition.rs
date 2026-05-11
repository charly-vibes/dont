use crate::store::{Store, StoreError};

use super::RuleMatch;

pub const EXPLANATION: &str = include_str!("dangling_definition.md");

/// Fires when a claim has a `term:uuid`-format dependency that can't be resolved by ID.
///
/// A dangling definition is an explicit ID reference to a term that no longer exists
/// (or was never created). Unlike `ungrounded` (which fires for unresolved CURIE-format
/// references), this rule fires for structurally broken `term:` ID references.
pub fn check(store: &Store) -> Result<Vec<RuleMatch>, StoreError> {
    let claims = store.list_claims()?;
    let mut matches = Vec::new();
    for claim in claims {
        for dep in &claim.depends_on {
            if !dep.starts_with("term:") {
                continue;
            }
            if store.term_by_id(dep)?.is_none() {
                matches.push(RuleMatch {
                    entity_id: claim.id.clone(),
                    detail: format!("depends on dangling term ID '{dep}' (no such term exists)"),
                });
            }
        }
    }
    Ok(matches)
}

#[cfg(test)]
mod dangling_definition {
    use tempfile::TempDir;

    use super::*;
    use crate::store::Store;

    fn make_store(dir: &TempDir) -> Store {
        Store::open_dont_dir(dir.path()).unwrap()
    }

    #[test]
    fn silent_when_term_id_dep_resolves() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let term = store.append_term("ns:concept", "", None).unwrap();
        store
            .append_claim("a claim", &[term.id.clone()])
            .unwrap();
        assert!(check(&store).unwrap().is_empty());
    }

    #[test]
    fn fires_when_term_id_dep_missing() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store
            .append_claim("a claim", &["term:nonexistent-id".to_string()])
            .unwrap();
        let matches = check(&store).unwrap();
        assert_eq!(matches.len(), 1);
        assert!(matches[0].detail.contains("term:nonexistent-id"));
    }

    #[test]
    fn does_not_fire_for_unresolved_curie() {
        // Unresolved CURIEs are handled by `ungrounded`, not `dangling_definition`.
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store
            .append_claim("a claim", &["ns:missing".to_string()])
            .unwrap();
        assert!(check(&store).unwrap().is_empty());
    }

    #[test]
    fn silent_when_no_deps() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store.append_claim("a claim", &[]).unwrap();
        assert!(check(&store).unwrap().is_empty());
    }
}
