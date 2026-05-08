use std::collections::BTreeSet;

use serde_json::Value;

use crate::store::{ClaimRecord, EventRecord, HypothesisRecord, Store, StoreError, StoreStatus};


use super::RuleMatch;

const MIN_ASSESSED_HYPOTHESES: usize = 3;
const MIN_INDEPENDENT_EVIDENCE: usize = 2;

/// Fires for claims that do not yet meet the gate conditions required before locking.
///
/// Conditions: ≥3 assessed hypotheses, ≥2 independent evidence sources, and no
/// derived assessments (stale, compromised-support) from dependencies.
pub fn check(store: &Store) -> Result<Vec<RuleMatch>, StoreError> {
    let claims = store.list_claims()?;
    let mut matches = Vec::new();
    for claim in &claims {
        let reasons = unmet_reasons(claim, store)?;
        if !reasons.is_empty() {
            matches.push(RuleMatch {
                entity_id: claim.id.clone(),
                detail: reasons.join("; "),
            });
        }
    }
    Ok(matches)
}

fn unmet_reasons(claim: &ClaimRecord, store: &Store) -> Result<Vec<String>, StoreError> {
    let mut reasons = Vec::new();

    let assessed = assessed_count(&claim.hypotheses);
    if assessed < MIN_ASSESSED_HYPOTHESES {
        reasons.push(format!(
            "needs >={MIN_ASSESSED_HYPOTHESES} assessed hypotheses; has {assessed}"
        ));
    }

    let evidence_count = independent_evidence_count(&claim.events);
    if evidence_count < MIN_INDEPENDENT_EVIDENCE {
        reasons.push(format!(
            "needs >={MIN_INDEPENDENT_EVIDENCE} independent evidence sources; has {evidence_count}"
        ));
    }

    for dep in &claim.depends_on {
        let term = if dep.starts_with("term:") {
            store.term_by_id(dep)?
        } else {
            store.term_by_curie(dep)?
        };
        if let Some(term) = term {
            match term.status {
                StoreStatus::Verified => {}
                _ => {
                    reasons.push(format!(
                        "dependency {} has blocking assessment {}",
                        term.id,
                        term.status.as_str()
                    ));
                }
            }
        }
    }

    Ok(reasons)
}

fn assessed_count(hypotheses: &[HypothesisRecord]) -> usize {
    hypotheses
        .iter()
        .filter(|h| {
            !h.assessment.supporting.is_empty() || !h.assessment.refuting.is_empty()
        })
        .count()
}

fn independent_evidence_count(events: &[EventRecord]) -> usize {
    let mut sources = BTreeSet::new();
    for ev in events {
        for entry in &ev.evidence {
            sources.insert(source_key(entry));
        }
    }
    sources.len()
}

fn source_key(v: &Value) -> String {
    if let Some(uri) = v.as_str() {
        return host_from_uri(uri);
    }
    if let Some(path) = v
        .as_object()
        .filter(|o| o.get("kind").and_then(Value::as_str) == Some("repo-file"))
        .and_then(|o| o.get("path"))
        .and_then(Value::as_str)
    {
        return format!("repo-file:{path}");
    }
    v.to_string()
}

fn host_from_uri(uri: &str) -> String {
    let without_scheme = uri.split_once("://").map(|(_, rest)| rest).unwrap_or(uri);
    let host = without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or(without_scheme)
        .trim();
    if host.is_empty() {
        uri.to_string()
    } else {
        host.to_lowercase()
    }
}

#[cfg(test)]
mod lockable {
    use tempfile::TempDir;

    use super::*;
    use crate::store::{Store, StoreEvent, StoreEventKind};

    fn make_store(dir: &TempDir) -> Store {
        Store::open_dont_dir(dir.path()).unwrap()
    }

    fn add_hypotheses(store: &Store, claim_id: &str) {
        for text in ["H1", "H2", "H3"] {
            let (_, idx) = store.add_hypothesis(claim_id, text).unwrap();
            store
                .assess_hypothesis(
                    claim_id,
                    idx,
                    &["https://example.com".to_string()],
                    &[],
                )
                .unwrap();
        }
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
    fn fires_when_hypotheses_below_threshold() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let result = store.append_claim("a claim", &[]).unwrap();
        let matches = check(&store).unwrap();
        assert!(
            matches.iter().any(|m| m.entity_id == result.id
                && m.detail.contains("assessed hypotheses")),
            "expected lockable to fire for insufficient hypotheses"
        );
    }

    #[test]
    fn fires_when_evidence_below_threshold() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let result = store.append_claim("a claim", &[]).unwrap();
        add_hypotheses(&store, &result.id);
        // Only one evidence source
        add_evidence(&store, &result.id, &["https://source-a.example.com/page"]);
        let matches = check(&store).unwrap();
        assert!(
            matches.iter().any(|m| m.entity_id == result.id
                && m.detail.contains("independent evidence")),
            "expected lockable to fire for correlated evidence"
        );
    }

    #[test]
    fn silent_when_all_conditions_met() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let result = store.append_claim("a claim", &[]).unwrap();
        add_hypotheses(&store, &result.id);
        add_evidence(
            &store,
            &result.id,
            &["https://source-a.example.com", "https://source-b.example.com"],
        );
        let matches = check(&store).unwrap();
        assert!(
            !matches.iter().any(|m| m.entity_id == result.id),
            "expected lockable to be silent when all conditions met"
        );
    }
}
