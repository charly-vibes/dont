use crate::store::{Store, StoreError};

use super::RuleMatch;

/// Fires when a claim depends on a CURIE reference that can't be resolved to a known term.
///
/// Only fires for CURIE-format deps (not `term:uuid` ID references — see `dangling_definition`).
pub fn check(store: &Store) -> Result<Vec<RuleMatch>, StoreError> {
    let claims = store.list_claims()?;
    let mut matches = Vec::new();
    for claim in claims {
        for dep in &claim.depends_on {
            if dep.starts_with("term:") {
                continue;
            }
            // Short-circuit for malformed deps (no ':') — always unresolvable.
            if !dep.contains(':') || store.term_by_curie(dep)?.is_none() {
                matches.push(RuleMatch {
                    entity_id: claim.id.clone(),
                    detail: format!("depends on unresolved term CURIE '{dep}'"),
                });
            }
        }
    }
    Ok(matches)
}

#[cfg(test)]
mod ungrounded {
    use tempfile::TempDir;

    use super::*;
    use crate::store::Store;

    fn make_store(dir: &TempDir) -> Store {
        Store::open_dont_dir(dir.path()).unwrap()
    }

    #[test]
    fn silent_when_all_curie_deps_resolve() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store.append_term("ns:concept", "", None).unwrap();
        store
            .append_claim("a claim", &["ns:concept".to_string()])
            .unwrap();
        assert!(check(&store).unwrap().is_empty());
    }

    #[test]
    fn fires_when_curie_dep_unresolved() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store
            .append_claim("a claim", &["ns:missing".to_string()])
            .unwrap();
        let matches = check(&store).unwrap();
        assert_eq!(matches.len(), 1);
        assert!(matches[0].detail.contains("ns:missing"));
    }

    #[test]
    fn silent_when_no_deps() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store.append_claim("a claim", &[]).unwrap();
        assert!(check(&store).unwrap().is_empty());
    }

    #[test]
    fn does_not_fire_for_term_id_deps() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store
            .append_claim("a claim", &["term:nonexistent-uuid".to_string()])
            .unwrap();
        assert!(check(&store).unwrap().is_empty());
    }

    #[test]
    fn fires_once_per_unresolved_curie() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store
            .append_claim(
                "a claim",
                &["ns:missing-a".to_string(), "ns:missing-b".to_string()],
            )
            .unwrap();
        let matches = check(&store).unwrap();
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn fires_for_malformed_dep_without_colon() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store
            .append_claim("a claim", &["malformed-no-colon".to_string()])
            .unwrap();
        let matches = check(&store).unwrap();
        assert_eq!(matches.len(), 1);
        assert!(matches[0].detail.contains("malformed-no-colon"));
    }
}
