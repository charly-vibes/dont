use crate::config::TermNonfunctionalConfig;
use crate::store::{Store, StoreError};

use super::RuleMatch;

pub const EXPLANATION: &str = include_str!("term_nonfunctional_label.md");

/// Fires when a term's label matches a nonfunctional-label pattern from the config.
///
/// Off by default (`config.enabled = false`). Pattern-driven via `[rules.term_nonfunctional]`.
/// Warn severity — does not block operations.
pub fn check(
    store: &Store,
    config: &TermNonfunctionalConfig,
) -> Result<Vec<RuleMatch>, StoreError> {
    if !config.enabled || config.patterns.is_empty() {
        return Ok(vec![]);
    }
    let terms = store.list_terms()?;
    let mut matches = Vec::new();
    for term in terms {
        if let Some(label) = &term.label {
            if config.matches_label(label) {
                matches.push(RuleMatch {
                    entity_id: term.id.clone(),
                    detail: format!(
                        "label '{label}' suggests a non-functional relationship; consider aspect-based decomposition"
                    ),
                });
            }
        }
    }
    Ok(matches)
}

#[cfg(test)]
mod term_nonfunctional_label {
    use tempfile::TempDir;

    use super::*;
    use crate::config::TermNonfunctionalConfig;
    use crate::store::Store;

    fn make_store(dir: &TempDir) -> Store {
        Store::open_dont_dir(dir.path()).unwrap()
    }

    fn enabled_config(patterns: &[&str]) -> TermNonfunctionalConfig {
        TermNonfunctionalConfig {
            enabled: true,
            patterns: patterns.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn silent_when_disabled() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store
            .append_term("ns:concept", "", Some("has-a dependency"))
            .unwrap();
        let config = TermNonfunctionalConfig {
            enabled: false,
            patterns: vec!["has-a".to_string()],
        };
        assert!(check(&store, &config).unwrap().is_empty());
    }

    #[test]
    fn silent_when_no_patterns() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store
            .append_term("ns:concept", "", Some("has-a dependency"))
            .unwrap();
        let config = TermNonfunctionalConfig {
            enabled: true,
            patterns: vec![],
        };
        assert!(check(&store, &config).unwrap().is_empty());
    }

    #[test]
    fn fires_when_label_matches_pattern() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store
            .append_term("ns:concept", "", Some("has-a dependency"))
            .unwrap();
        let config = enabled_config(&["has-a"]);
        let matches = check(&store, &config).unwrap();
        assert_eq!(matches.len(), 1);
        assert!(matches[0].detail.contains("has-a dependency"));
    }

    #[test]
    fn silent_when_label_does_not_match() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store
            .append_term("ns:concept", "", Some("functional label"))
            .unwrap();
        let config = enabled_config(&["has-a", "is-a"]);
        assert!(check(&store, &config).unwrap().is_empty());
    }

    #[test]
    fn silent_when_term_has_no_label() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        store.append_term("ns:concept", "", None).unwrap();
        let config = enabled_config(&["has-a"]);
        assert!(check(&store, &config).unwrap().is_empty());
    }
}
