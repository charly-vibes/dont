use crate::store::{Store, StoreError};

use super::RuleMatch;

pub const EXPLANATION: &str = include_str!("unresolved_terms.md");

/// Fires when any dependency reference (CURIE or `term:uuid`) can't be resolved.
///
/// Always strict — blocks the verify/dismiss operation.
pub fn check(store: &Store) -> Result<Vec<RuleMatch>, StoreError> {
    let claims = store.list_claims()?;
    let mut matches = Vec::new();
    for claim in claims {
        for dep in &claim.depends_on {
            let resolved = if dep.starts_with("term:") {
                store.term_by_id(dep)?.map(crate::store::CurieResolution::Coined)
            } else {
                store.resolve_curie_reference(dep)?
            };
            if resolved.is_none() {
                matches.push(RuleMatch {
                    entity_id: claim.id.clone(),
                    detail: format!("depends on unresolved term reference '{dep}'"),
                });
            }
        }
    }
    Ok(matches)
}

#[cfg(test)]
mod unresolved_terms_tests {
    use tempfile::TempDir;

    use super::*;
    use crate::store::Store;

    fn make_store(dir: &TempDir) -> Store {
        Store::open_dont_dir(dir.path()).unwrap()
    }

    #[test]
    fn silent_when_all_deps_resolve() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let term = store.append_term("ns:concept", "", None).unwrap();
        store
            .append_claim("a claim", &["ns:concept".to_string(), term.id.clone()], None)
            .unwrap();
        assert!(check(&store).unwrap().is_empty());
    }

    #[test]
    fn fires_for_unresolved_curie() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store
            .append_claim("a claim", &["ns:missing".to_string()], None)
            .unwrap();
        let matches = check(&store).unwrap();
        assert_eq!(matches.len(), 1);
        assert!(matches[0].detail.contains("ns:missing"));
    }

    #[test]
    fn fires_for_unresolved_term_id() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store
            .append_claim("a claim", &["term:nonexistent-uuid".to_string()], None)
            .unwrap();
        let matches = check(&store).unwrap();
        assert_eq!(matches.len(), 1);
        assert!(matches[0].detail.contains("term:nonexistent-uuid"));
    }

    #[test]
    fn silent_when_no_deps() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store.append_claim("a claim", &[], None).unwrap();
        assert!(check(&store).unwrap().is_empty());
    }
}
