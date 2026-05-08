use crate::config::RuleClaimStructureConfig;
use crate::store::{Store, StoreError};

use super::RuleMatch;

/// Fires when a tagged rule claim is missing mandatory slot markers.
///
/// Off by default (`config.enabled = false`). Requires `tag_term_id` to be set
/// (the `term:uuid` of the `rule-claim-type` anchor term). Warn severity —
/// does not block operations or change stored claim status.
pub fn check(
    store: &Store,
    config: &RuleClaimStructureConfig,
) -> Result<Vec<RuleMatch>, StoreError> {
    let tag = match config.enabled_tag() {
        Some(t) => t,
        None => return Ok(vec![]),
    };
    let claims = store.list_claims()?;
    let mut matches = Vec::new();
    for claim in claims {
        if !claim.depends_on.iter().any(|d| d == tag) {
            continue;
        }
        let stmt = &claim.statement;
        if !stmt.contains("[TRIGGER]") {
            matches.push(RuleMatch {
                entity_id: claim.id.clone(),
                detail: "tagged rule claim is missing mandatory [TRIGGER] slot".to_string(),
            });
        }
        if !stmt.contains("[CONFIG]") && !stmt.contains("[MODE]") {
            matches.push(RuleMatch {
                entity_id: claim.id.clone(),
                detail: "tagged rule claim is missing mandatory CONFIG/MODE slot — include at least one of [CONFIG] or [MODE]".to_string(),
            });
        }
    }
    Ok(matches)
}

#[cfg(test)]
mod rule_claim_structure {
    use tempfile::TempDir;

    use super::*;
    use crate::config::RuleClaimStructureConfig;
    use crate::store::Store;

    fn make_store(dir: &TempDir) -> Store {
        Store::open_dont_dir(dir.path()).unwrap()
    }

    fn enabled_config(tag: &str) -> RuleClaimStructureConfig {
        RuleClaimStructureConfig {
            enabled: true,
            tag_term_id: Some(tag.to_string()),
        }
    }

    #[test]
    fn silent_when_disabled() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let term = store.append_term("ns:rule-claim-type", "", None).unwrap();
        store
            .append_claim("[CONFIG] Enabled: yes", &[term.id.clone()])
            .unwrap();
        let config = RuleClaimStructureConfig {
            enabled: false,
            tag_term_id: Some(term.id.clone()),
        };
        assert!(check(&store, &config).unwrap().is_empty());
    }

    #[test]
    fn silent_when_no_tag_term_id() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let term = store.append_term("ns:rule-claim-type", "", None).unwrap();
        store
            .append_claim("no trigger or mode here", &[term.id.clone()])
            .unwrap();
        let config = RuleClaimStructureConfig {
            enabled: true,
            tag_term_id: None,
        };
        assert!(check(&store, &config).unwrap().is_empty());
    }

    #[test]
    fn silent_when_claim_not_tagged() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let term = store.append_term("ns:rule-claim-type", "", None).unwrap();
        store.append_claim("no trigger, no mode", &[]).unwrap();
        let config = enabled_config(&term.id);
        assert!(check(&store, &config).unwrap().is_empty());
    }

    #[test]
    fn fires_when_trigger_missing() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let term = store.append_term("ns:rule-claim-type", "", None).unwrap();
        store
            .append_claim("[CONFIG] Enabled by default: yes", &[term.id.clone()])
            .unwrap();
        let config = enabled_config(&term.id);
        let matches = check(&store, &config).unwrap();
        assert_eq!(matches.len(), 1);
        assert!(matches[0].detail.contains("[TRIGGER]"));
    }

    #[test]
    fn fires_when_config_and_mode_both_missing() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let term = store.append_term("ns:rule-claim-type", "", None).unwrap();
        store
            .append_claim("[TRIGGER] fires when X happens", &[term.id.clone()])
            .unwrap();
        let config = enabled_config(&term.id);
        let matches = check(&store, &config).unwrap();
        assert_eq!(matches.len(), 1);
        let detail = &matches[0].detail;
        assert!(detail.contains("[CONFIG]") || detail.contains("[MODE]"));
    }

    #[test]
    fn fires_two_violations_when_both_mandatory_slots_missing() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let term = store.append_term("ns:rule-claim-type", "", None).unwrap();
        store
            .append_claim("[GUARD] silently skips empty inputs", &[term.id.clone()])
            .unwrap();
        let config = enabled_config(&term.id);
        let matches = check(&store, &config).unwrap();
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn silent_when_mandatory_slots_present_with_config() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let term = store.append_term("ns:rule-claim-type", "", None).unwrap();
        store
            .append_claim(
                "[TRIGGER] fires when X\n[CONFIG] Enabled by default: yes",
                &[term.id.clone()],
            )
            .unwrap();
        let config = enabled_config(&term.id);
        assert!(check(&store, &config).unwrap().is_empty());
    }

    #[test]
    fn silent_when_mandatory_slots_present_with_mode() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let term = store.append_term("ns:rule-claim-type", "", None).unwrap();
        store
            .append_claim(
                "[TRIGGER] fires when Y\n[MODE] In permissive mode: warn",
                &[term.id.clone()],
            )
            .unwrap();
        let config = enabled_config(&term.id);
        assert!(check(&store, &config).unwrap().is_empty());
    }

    #[test]
    fn mode_alone_satisfies_config_mode_requirement() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let term = store.append_term("ns:rule-claim-type", "", None).unwrap();
        store
            .append_claim(
                "[TRIGGER] fires when Z\n[MODE] warn in all modes",
                &[term.id.clone()],
            )
            .unwrap();
        let config = enabled_config(&term.id);
        assert!(check(&store, &config).unwrap().is_empty());
    }

    #[test]
    fn only_tagged_claims_are_evaluated() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let term = store.append_term("ns:rule-claim-type", "", None).unwrap();
        // Tagged claim: complete
        store
            .append_claim(
                "[TRIGGER] complete\n[CONFIG] yes",
                &[term.id.clone()],
            )
            .unwrap();
        // Untagged claim: would violate if evaluated
        store.append_claim("no trigger, no mode", &[]).unwrap();
        let config = enabled_config(&term.id);
        assert!(check(&store, &config).unwrap().is_empty());
    }
}
