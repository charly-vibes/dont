use std::collections::BTreeSet;

use crate::store::{ClaimRecord, CurieResolution, EventRecord, HypothesisRecord, Status, Store, StoreError};

use super::{source_key, RuleMatch};

pub const EXPLANATION: &str = include_str!("lockable.md");

const MIN_ASSESSED_HYPOTHESES: usize = 3;
const MIN_INDEPENDENT_EVIDENCE: usize = 2;

/// Fires for claims that do not yet meet the gate conditions required before locking.
///
/// Intended as a pre-lock gate check, not a background lint — run with
/// `dont check --lock-readiness`.
///
/// Conditions: ≥3 assessed hypotheses, ≥2 independent evidence sources, and all
/// dependencies verified or resolvable.
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

    if !matches!(claim.status, Status::Verified) {
        reasons.push(format!(
            "claim must be in verified status before locking; has {}",
            claim.status.as_str()
        ));
    }

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
            store.term_by_id(dep)?.map(CurieResolution::Coined)
        } else {
            store.resolve_curie_reference(dep)?
        };
        match term {
            Some(CurieResolution::Coined(term)) => match term.status {
                Status::Verified => {}
                _ => {
                    reasons.push(format!(
                        "dependency {} has blocking assessment {}",
                        term.id,
                        term.status.as_str()
                    ));
                }
            },
            Some(CurieResolution::Imported(_)) => {}
            None => reasons.push(format!("dependency {dep} is unresolved")),
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

#[cfg(test)]
mod lockable_tests {
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
        let result = store.append_claim("a claim", &[], None).unwrap();
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
        let result = store.append_claim("a claim", &[], None).unwrap();
        add_hypotheses(&store, &result.id);
        // Only one evidence source
        add_evidence(&store, &result.id, &["https://source-a.example.com/page"]);
        let matches = check(&store).unwrap();
        assert!(
            matches.iter().any(|m| m.entity_id == result.id
                && m.detail.contains("independent evidence")),
            "expected lockable to fire for insufficient evidence sources"
        );
    }

    #[test]
    fn fires_for_unresolved_dep() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let result = store
            .append_claim("a claim", &["ns:missing".to_string()], None)
            .unwrap();
        add_hypotheses(&store, &result.id);
        add_evidence(
            &store,
            &result.id,
            &["https://source-a.example.com", "https://source-b.example.com"],
        );
        let matches = check(&store).unwrap();
        assert!(
            matches.iter().any(|m| m.entity_id == result.id
                && m.detail.contains("is unresolved")),
            "expected lockable to fire for unresolved dependency"
        );
    }

    #[test]
    fn fires_when_claim_is_unverified_despite_other_conditions_met() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let result = store.append_claim("a claim", &[], None).unwrap();
        add_hypotheses(&store, &result.id);
        add_evidence(
            &store,
            &result.id,
            &["https://source-a.example.com", "https://source-b.example.com"],
        );
        // Intentionally NOT verifying — claim stays Unverified
        let matches = check(&store).unwrap();
        assert!(
            matches.iter().any(|m| m.entity_id == result.id
                && m.detail.to_lowercase().contains("verified")),
            "lockable must fire when claim is not in verified status; got: {:?}",
            matches.iter().find(|m| m.entity_id == result.id)
        );
    }

    #[test]
    fn silent_when_all_conditions_met() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let result = store.append_claim("a claim", &[], None).unwrap();
        add_hypotheses(&store, &result.id);
        add_evidence(
            &store,
            &result.id,
            &["https://source-a.example.com", "https://source-b.example.com"],
        );
        store
            .append_status_change(
                &result.id,
                Status::Unverified,
                Status::Verified,
                StoreEvent { kind: StoreEventKind::Flagged, note: None, evidence: vec![] },
            )
            .unwrap();
        let matches = check(&store).unwrap();
        assert!(
            !matches.iter().any(|m| m.entity_id == result.id),
            "expected lockable to be silent when all conditions met"
        );
    }

    // Spec condition 1: the claim must be in :verified status.
    // Claims not yet verified are not candidates for locking — lockable should be silent.
    #[test]
    fn silent_for_unverified_claim() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        // Claim stays in default :unverified status
        let result = store.append_claim("an unverified claim", &[], None).unwrap();
        let matches = check(&store).unwrap();
        assert!(
            !matches.iter().any(|m| m.entity_id == result.id),
            "lockable should be silent for claims not in :verified status"
        );
    }

    // Spec condition 1: the claim must be in :verified status.
    // A doubted claim is not a lock candidate.
    #[test]
    fn silent_for_doubted_claim() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let result = store.append_claim("a doubted claim", &[], None).unwrap();
        store
            .append_status_change(
                &result.id,
                Status::Unverified,
                Status::Doubted,
                StoreEvent {
                    kind: StoreEventKind::Flagged,
                    note: None,
                    evidence: vec![],
                },
            )
            .unwrap();
        let matches = check(&store).unwrap();
        assert!(
            !matches.iter().any(|m| m.entity_id == result.id),
            "lockable should be silent for claims in :doubted status"
        );
    }

    // Spec condition 1: the claim must be in :verified status.
    // A verified claim that lacks other conditions SHOULD trigger lockable.
    #[test]
    fn fires_for_verified_claim_missing_hypotheses() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let result = store.append_claim("a verified claim", &[], None).unwrap();
        store
            .append_status_change(
                &result.id,
                Status::Unverified,
                Status::Verified,
                StoreEvent {
                    kind: StoreEventKind::Trusted,
                    note: None,
                    evidence: vec![],
                },
            )
            .unwrap();
        let matches = check(&store).unwrap();
        assert!(
            matches.iter().any(|m| m.entity_id == result.id
                && m.detail.contains("assessed hypotheses")),
            "lockable should fire for a :verified claim lacking sufficient hypotheses"
        );
    }
}
