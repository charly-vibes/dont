use std::collections::HashMap;

use serde_json::Value;

use crate::store::{EventRecord, Store, StoreError};

pub const EXPLANATION: &str = include_str!("correlated_error.md");

use super::{RuleMatch, source_key};

/// Fires when a claim's evidence items come from fewer unique sources than the total count,
/// indicating that some evidence is from correlated (shared) sources.
pub fn check(store: &Store) -> Result<Vec<RuleMatch>, StoreError> {
    let claims = store.list_claims()?;
    let mut matches = Vec::new();
    for claim in claims {
        let evidence = collect_evidence(&claim.events);
        if evidence.len() < 2 {
            continue;
        }
        let mut source_counts: HashMap<String, usize> = HashMap::new();
        for entry in &evidence {
            *source_counts.entry(source_key(entry)).or_default() += 1;
        }
        let correlated: Vec<String> = source_counts
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(source, count)| format!("{source} ({count} items)"))
            .collect();
        if !correlated.is_empty() {
            matches.push(RuleMatch {
                entity_id: claim.id.clone(),
                detail: format!(
                    "evidence from correlated sources: {}",
                    correlated.join(", ")
                ),
            });
        }
    }
    Ok(matches)
}

fn collect_evidence(events: &[EventRecord]) -> Vec<Value> {
    events
        .iter()
        .flat_map(|ev| ev.evidence.iter().cloned())
        .collect()
}

#[cfg(test)]
mod correlated_error_tests {
    use tempfile::TempDir;

    use super::*;
    use crate::store::{Store, StoreEvent, StoreEventKind};

    fn make_store(dir: &TempDir) -> Store {
        Store::open_dont_dir(dir.path()).unwrap()
    }

    fn add_evidence(store: &Store, claim_id: &str, uris: &[&str]) {
        for uri in uris {
            store
                .append_evidence_event(
                    claim_id,
                    StoreEvent {
                        kind: StoreEventKind::Flagged,
                        note: None,
                        evidence: vec![serde_json::Value::String((*uri).to_string())],
                    },
                )
                .unwrap();
        }
    }

    #[test]
    fn silent_when_evidence_from_independent_sources() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let result = store.append_claim("a claim", &[], None).unwrap();
        add_evidence(
            &store,
            &result.id,
            &[
                "https://source-a.example.com/page",
                "https://source-b.example.com/page",
            ],
        );
        assert!(check(&store).unwrap().is_empty());
    }

    #[test]
    fn fires_when_evidence_from_same_host() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let result = store.append_claim("a claim", &[], None).unwrap();
        add_evidence(
            &store,
            &result.id,
            &[
                "https://same-source.example.com/page-1",
                "https://same-source.example.com/page-2",
            ],
        );
        let matches = check(&store).unwrap();
        assert_eq!(matches.len(), 1);
        assert!(matches[0].detail.contains("same-source.example.com"));
    }

    #[test]
    fn silent_when_single_evidence_item() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let result = store.append_claim("a claim", &[], None).unwrap();
        add_evidence(&store, &result.id, &["https://example.com/only-one"]);
        assert!(check(&store).unwrap().is_empty());
    }

    #[test]
    fn silent_when_no_evidence() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store.append_claim("a claim", &[], None).unwrap();
        assert!(check(&store).unwrap().is_empty());
    }

    #[test]
    fn terminates_on_claim_with_no_cyclic_deps() {
        // Claims only depend on terms — no cyclic graph is possible in current model.
        // This test verifies the rule evaluates without hanging.
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store.append_term("ns:concept", "", None).unwrap();
        let result = store
            .append_claim("a claim", &["ns:concept".to_string()], None)
            .unwrap();
        add_evidence(
            &store,
            &result.id,
            &["https://a.example.com", "https://b.example.com"],
        );
        assert!(check(&store).unwrap().is_empty());
    }
}
