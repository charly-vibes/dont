use crate::store::{ClaimRecord, Status, Store, StoreError, StoreEventKind};

use super::RuleMatch;

pub const EXPLANATION: &str = include_str!("stale_cascade.md");

/// Fires when a verified claim has a hypothesis assessment recorded before
/// supporting evidence was later attached to the claim.
///
/// "Stale" is derived on demand — this rule does not persist any status.
pub fn check(store: &Store) -> Result<Vec<RuleMatch>, StoreError> {
    let claims = store.list_claims()?;
    let mut matches = Vec::new();
    for claim in claims {
        if !matches!(claim.status, Status::Verified) {
            continue;
        }
        if is_stale(&claim) {
            matches.push(RuleMatch {
                entity_id: claim.id.clone(),
                detail: "has an assessment recorded before supporting evidence was attached"
                    .to_string(),
            });
        }
    }
    Ok(matches)
}

/// Returns true when the claim has a hypothesis assessment whose tx is less
/// than the tx of some later evidence event — meaning the assessment was made
/// before that evidence was available.
fn is_stale(claim: &ClaimRecord) -> bool {
    let min_assessed_tx = claim
        .events
        .iter()
        .filter(|e| e.kind == StoreEventKind::HypothesisAssessed)
        .map(|e| e.tx)
        .min();

    let Some(min_assessed_tx) = min_assessed_tx else {
        return false;
    };

    claim
        .events
        .iter()
        .filter(|e| !e.evidence.is_empty())
        .any(|e| e.tx > min_assessed_tx)
}

#[cfg(test)]
mod stale_cascade_tests {
    use tempfile::TempDir;

    use super::*;
    use crate::store::{Store, StoreEvent, StoreEventKind};

    fn make_store(dir: &TempDir) -> Store {
        Store::open_dont_dir(dir.path()).unwrap()
    }

    fn verify_claim(store: &Store, claim_id: &str) {
        store
            .append_status_change(
                claim_id,
                Status::Unverified,
                Status::Verified,
                StoreEvent {
                    kind: StoreEventKind::Trusted,
                    note: None,
                    evidence: vec![],
                },
            )
            .unwrap();
    }

    fn assess_hypothesis(store: &Store, claim_id: &str) {
        let (_, idx) = store.add_hypothesis(claim_id, "H1").unwrap();
        store
            .assess_hypothesis(claim_id, idx, &["https://example.com/a".to_string()], &[])
            .unwrap();
    }

    fn add_evidence(store: &Store, claim_id: &str) {
        store
            .append_evidence_event(
                claim_id,
                StoreEvent {
                    kind: StoreEventKind::Flagged,
                    note: None,
                    evidence: vec![serde_json::json!("https://evidence.example.com")],
                },
            )
            .unwrap();
    }

    #[test]
    fn fires_when_evidence_added_after_hypothesis_assessment() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let claim = store.append_claim("a claim", &[], None).unwrap();
        verify_claim(&store, &claim.id);
        assess_hypothesis(&store, &claim.id);
        add_evidence(&store, &claim.id); // evidence after assessment → stale
        let matches = check(&store).unwrap();
        assert!(
            matches.iter().any(|m| m.entity_id == claim.id),
            "stale-cascade must fire when evidence is added after a hypothesis assessment"
        );
    }

    #[test]
    fn silent_when_assessment_made_after_evidence() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let claim = store.append_claim("a claim", &[], None).unwrap();
        verify_claim(&store, &claim.id);
        add_evidence(&store, &claim.id); // evidence first
        assess_hypothesis(&store, &claim.id); // assessment after → not stale
        let matches = check(&store).unwrap();
        assert!(
            !matches.iter().any(|m| m.entity_id == claim.id),
            "stale-cascade must be silent when all evidence predates the assessment"
        );
    }

    #[test]
    fn silent_for_claim_with_no_assessed_hypotheses() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let claim = store.append_claim("a claim", &[], None).unwrap();
        verify_claim(&store, &claim.id);
        add_evidence(&store, &claim.id); // evidence but no hypothesis assessment
        assert!(
            !check(&store)
                .unwrap()
                .iter()
                .any(|m| m.entity_id == claim.id),
            "stale-cascade must be silent with no hypothesis assessments"
        );
    }

    #[test]
    fn silent_for_unverified_claim_with_post_assessment_evidence() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let claim = store.append_claim("a claim", &[], None).unwrap();
        // NOT verified — stays Unverified
        assess_hypothesis(&store, &claim.id);
        add_evidence(&store, &claim.id);
        assert!(
            !check(&store)
                .unwrap()
                .iter()
                .any(|m| m.entity_id == claim.id),
            "stale-cascade must only fire for verified claims"
        );
    }

    // Locked claims are no longer Verified, so stale-cascade is naturally exempt.
    #[test]
    fn silent_for_locked_claim_with_stale_pattern() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let claim = store.append_claim("a locked claim", &[], None).unwrap();
        verify_claim(&store, &claim.id);
        assess_hypothesis(&store, &claim.id);
        add_evidence(&store, &claim.id); // would be stale if Verified
        store
            .append_status_change(
                &claim.id,
                Status::Verified,
                Status::Locked,
                StoreEvent {
                    kind: StoreEventKind::Locked,
                    note: None,
                    evidence: vec![],
                },
            )
            .unwrap();
        assert!(
            !check(&store)
                .unwrap()
                .iter()
                .any(|m| m.entity_id == claim.id),
            "locked claim should be exempt from stale-cascade"
        );
    }

    // Ignored claims are not Verified, so stale-cascade is naturally exempt.
    #[test]
    fn silent_for_ignored_claim_with_stale_pattern() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let claim = store.append_claim("an ignored claim", &[], None).unwrap();
        // Not verified before ignoring
        assess_hypothesis(&store, &claim.id);
        add_evidence(&store, &claim.id);
        store
            .append_status_change(
                &claim.id,
                Status::Unverified,
                Status::Ignored,
                StoreEvent {
                    kind: StoreEventKind::Ignored,
                    note: None,
                    evidence: vec![],
                },
            )
            .unwrap();
        assert!(
            !check(&store)
                .unwrap()
                .iter()
                .any(|m| m.entity_id == claim.id),
            "ignored claim should be exempt from stale-cascade"
        );
    }
}
