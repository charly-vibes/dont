use std::path::PathBuf;

use serde_json::Value;

use crate::config::RulesConfig;
use crate::store::{Store, StoreError};

pub mod correlated_error;
pub mod dangling_definition;
pub mod lockable;
pub mod rule_claim_structure;
pub mod stale_cascade;
pub mod term_nonfunctional_label;
pub mod ungrounded;
pub mod unresolved_terms;

pub const SHIPPED_RULES: &[&str] = &[
    "ungrounded",
    "unresolved-terms",
    "stale-cascade",
    "lockable",
    "correlated-error",
    "dangling-definition",
    "term-nonfunctional-label",
    "rule-claim-structure",
];

/// Returns the embedded prose explanation for a shipped rule, or `None` if unknown.
pub fn explain(rule_name: &str) -> Option<&'static str> {
    match rule_name {
        "ungrounded" => Some(ungrounded::EXPLANATION),
        "unresolved-terms" => Some(unresolved_terms::EXPLANATION),
        "stale-cascade" => Some(stale_cascade::EXPLANATION),
        "lockable" => Some(lockable::EXPLANATION),
        "correlated-error" => Some(correlated_error::EXPLANATION),
        "dangling-definition" => Some(dangling_definition::EXPLANATION),
        "term-nonfunctional-label" => Some(term_nonfunctional_label::EXPLANATION),
        "rule-claim-structure" => Some(rule_claim_structure::EXPLANATION),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct RuleMatch {
    pub entity_id: String,
    pub detail: String,
}

#[derive(Debug)]
pub enum RuleError {
    Io(std::io::Error),
    Compile { rule_name: String, message: String },
    Store(StoreError),
}

impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error reading rule: {e}"),
            Self::Compile { rule_name, message } => {
                write!(f, "rule {rule_name:?} failed to compile: {message}")
            }
            Self::Store(e) => write!(f, "store error evaluating rule: {e}"),
        }
    }
}

impl std::error::Error for RuleError {}

/// Effective severity for a rule: strict fires as refusal; warn fires as warning entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Warn,
    Strict,
}

pub struct RuleEngine {
    rules_dir: PathBuf,
    config: RulesConfig,
    mode_is_strict: bool,
}

impl RuleEngine {
    pub fn new(rules_dir: PathBuf, config: RulesConfig, mode_is_strict: bool) -> Self {
        Self {
            rules_dir,
            config,
            mode_is_strict,
        }
    }

    /// Effective severity for `rule_name`, respecting non-overridable boundaries.
    pub fn severity(&self, rule_name: &str) -> Severity {
        // Non-overridable rules are always strict regardless of config.
        if matches!(
            rule_name,
            "unresolved-terms" | "dangling-definition" | "stale-cascade"
        ) {
            return Severity::Strict;
        }
        // Explicit project-level config overrides (for overridable rules only).
        // strict takes precedence if a rule appears in both lists.
        if self.config.strict.iter().any(|r| r == rule_name) {
            return Severity::Strict;
        }
        if self.config.warn.iter().any(|r| r == rule_name) {
            return Severity::Warn;
        }
        // Mode-driven defaults.
        match rule_name {
            "ungrounded" if self.mode_is_strict => Severity::Strict,
            _ => Severity::Warn,
        }
    }

    /// Evaluate a shipped (built-in) rule by name.
    ///
    /// Returns `None` if `rule_name` is not a shipped rule; the caller should then fall
    /// back to file-based evaluation via [`Self::evaluate`].
    pub fn evaluate_shipped(
        &self,
        store: &Store,
        rule_name: &str,
    ) -> Option<Result<Vec<RuleMatch>, RuleError>> {
        let result = match rule_name {
            "ungrounded" => ungrounded::check(store),
            "unresolved-terms" => unresolved_terms::check(store),
            "stale-cascade" => stale_cascade::check(store),
            "lockable" => lockable::check(store),
            "correlated-error" => correlated_error::check(store),
            "dangling-definition" => dangling_definition::check(store),
            "term-nonfunctional-label" => {
                term_nonfunctional_label::check(store, &self.config.term_nonfunctional)
            }
            "rule-claim-structure" => {
                rule_claim_structure::check(store, &self.config.rule_claim_structure)
            }
            _ => return None,
        };
        Some(result.map_err(RuleError::Store))
    }

    /// Load `<rule_name>.dl` from the rules directory and evaluate it against `store`.
    ///
    /// Rules MUST be violation queries whose result columns are `[entity_id, detail]`.
    /// An empty result set means the rule is satisfied; non-empty rows are violations.
    ///
    /// Returns `RuleError::Compile` when the script fails to parse or execute.
    pub fn evaluate(&self, store: &Store, rule_name: &str) -> Result<Vec<RuleMatch>, RuleError> {
        let path = self.rules_dir.join(format!("{rule_name}.dl"));
        let script = std::fs::read_to_string(&path).map_err(RuleError::Io)?;
        let rows = store
            .run_rule_query(&script)
            .map_err(|e| RuleError::Compile {
                rule_name: rule_name.to_string(),
                message: e.to_string(),
            })?;
        rows_to_matches(rows, rule_name)
    }
}

fn rows_to_matches(rows: Vec<Vec<Value>>, rule_name: &str) -> Result<Vec<RuleMatch>, RuleError> {
    rows.into_iter()
        .enumerate()
        .map(|(i, row)| {
            let entity_id = row
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| RuleError::Compile {
                    rule_name: rule_name.to_string(),
                    message: format!("row {i}: entity_id must be a string"),
                })?
                .to_string();
            let detail = row
                .get(1)
                .and_then(Value::as_str)
                .ok_or_else(|| RuleError::Compile {
                    rule_name: rule_name.to_string(),
                    message: format!("row {i}: detail must be a string"),
                })?
                .to_string();
            Ok(RuleMatch { entity_id, detail })
        })
        .collect()
}

/// Returns a stable source key for deduplicating evidence entries across rules.
///
/// URI entries normalize to host (port stripped); repo-file objects use their
/// path; anything else falls back to the JSON representation.
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
        // Strip port so example.com and example.com:8080 count as the same source.
        host.split(':').next().unwrap_or(host).to_lowercase()
    }
}

#[cfg(test)]
mod engine {
    use std::fs;

    use tempfile::TempDir;

    use super::*;
    use crate::config::RulesConfig;
    use crate::store::Store;

    fn make_store(dir: &TempDir) -> Store {
        Store::open_dont_dir(dir.path()).unwrap()
    }

    fn make_engine(dir: &TempDir, config: RulesConfig, strict_mode: bool) -> RuleEngine {
        let rules_dir = dir.path().join("rules");
        fs::create_dir_all(&rules_dir).unwrap();
        RuleEngine::new(rules_dir, config, strict_mode)
    }

    fn write_rule(dir: &TempDir, name: &str, script: &str) {
        let path = dir.path().join("rules").join(format!("{name}.dl"));
        fs::write(path, script).unwrap();
    }

    #[test]
    fn rule_file_evaluates_against_store() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let engine = make_engine(&dir, RulesConfig::default(), false);

        write_rule(
            &dir,
            "always-fires",
            r#"?[entity_id, detail] <- [["test-entity", "test violation"]]"#,
        );

        let matches = engine.evaluate(&store, "always-fires").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].entity_id, "test-entity");
        assert_eq!(matches[0].detail, "test violation");
    }

    #[test]
    fn empty_result_when_no_violations() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let engine = make_engine(&dir, RulesConfig::default(), false);

        write_rule(
            &dir,
            "never-fires",
            r#"?[entity_id, detail] := *datoms[entity_id, "impossible_attr_xyz", detail, _, _]"#,
        );

        let matches = engine.evaluate(&store, "never-fires").unwrap();
        assert!(matches.is_empty());
    }

    #[test]
    fn invalid_dl_emits_compile_error() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let engine = make_engine(&dir, RulesConfig::default(), false);

        write_rule(&dir, "bad-syntax", "this is not valid datalog !!!@#$");

        let err = engine.evaluate(&store, "bad-syntax").unwrap_err();
        match err {
            RuleError::Compile { rule_name, .. } => assert_eq!(rule_name, "bad-syntax"),
            RuleError::Io(_) => panic!("expected compile error, got IO error"),
            RuleError::Store(_) => panic!("expected compile error, got store error"),
        }
    }

    #[test]
    fn strict_rule_fires_as_strict_severity() {
        let dir = TempDir::new().unwrap();
        let engine = make_engine(&dir, RulesConfig::default(), false);

        assert_eq!(engine.severity("unresolved-terms"), Severity::Strict);
        assert_eq!(engine.severity("dangling-definition"), Severity::Strict);
        assert_eq!(engine.severity("stale-cascade"), Severity::Strict);
    }

    #[test]
    fn warn_rules_default_to_warn_severity() {
        let dir = TempDir::new().unwrap();
        let engine = make_engine(&dir, RulesConfig::default(), false);

        assert_eq!(engine.severity("correlated-error"), Severity::Warn);
        assert_eq!(engine.severity("ungrounded"), Severity::Warn);
        assert_eq!(engine.severity("rule-claim-structure"), Severity::Warn);
    }

    #[test]
    fn rule_claim_structure_dispatches_from_evaluate_shipped() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let engine = make_engine(&dir, RulesConfig::default(), false);
        let result = engine.evaluate_shipped(&store, "rule-claim-structure");
        assert!(result.is_some(), "rule-claim-structure should be a shipped rule");
        assert!(result.unwrap().unwrap().is_empty());
    }

    #[test]
    fn strict_mode_escalates_ungrounded() {
        let dir = TempDir::new().unwrap();
        let engine = make_engine(&dir, RulesConfig::default(), true);

        assert_eq!(engine.severity("ungrounded"), Severity::Strict);
    }

    #[test]
    fn config_override_changes_severity() {
        let dir = TempDir::new().unwrap();
        let config = RulesConfig {
            strict: vec!["correlated-error".to_string()],
            warn: vec!["ungrounded".to_string()],
            ..RulesConfig::default()
        };
        let engine = make_engine(&dir, config, true);

        // Config warn override beats mode-driven strict for ungrounded.
        assert_eq!(engine.severity("ungrounded"), Severity::Warn);
        // Config strict override applied.
        assert_eq!(engine.severity("correlated-error"), Severity::Strict);
    }

    #[test]
    fn non_overridable_rules_ignore_config_override() {
        let dir = TempDir::new().unwrap();
        let config = RulesConfig {
            warn: vec![
                "unresolved-terms".to_string(),
                "dangling-definition".to_string(),
                "stale-cascade".to_string(),
            ],
            ..RulesConfig::default()
        };
        let engine = make_engine(&dir, config, false);

        // Non-overridable rules stay strict even if placed in warn list.
        assert_eq!(engine.severity("unresolved-terms"), Severity::Strict);
        assert_eq!(engine.severity("dangling-definition"), Severity::Strict);
        assert_eq!(engine.severity("stale-cascade"), Severity::Strict);
    }

    #[test]
    fn rule_queries_live_store_data() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let engine = make_engine(&dir, RulesConfig::default(), false);

        // Insert a claim and verify the rule can see it via datoms.
        let result = store.append_claim("a verifiable claim", &[], None).unwrap();
        let claim_id = result.id.clone();

        // Rule that finds all entities with a "statement" attribute.
        write_rule(
            &dir,
            "has-statement",
            r#"?[entity_id, detail] := *datoms[entity_id, "statement", _, _, true], detail = "found""#,
        );

        let matches = engine.evaluate(&store, "has-statement").unwrap();
        assert!(
            matches.iter().any(|m| m.entity_id == claim_id),
            "rule should find the inserted claim {claim_id}"
        );
    }

    /// Invariant: every name in SHIPPED_RULES must be dispatchable via evaluate_shipped.
    ///
    /// This guards against SHIPPED_RULES and the evaluate_shipped match arm drifting apart —
    /// e.g. a rule added to the list but forgotten in the dispatch table.
    #[test]
    fn all_shipped_rules_dispatch_via_evaluate_shipped() {
        let dir = TempDir::new().unwrap();
        let store = make_store(&dir);
        let engine = make_engine(&dir, RulesConfig::default(), false);

        for rule in SHIPPED_RULES {
            let result = engine.evaluate_shipped(&store, rule);
            assert!(
                result.is_some(),
                "SHIPPED_RULES contains '{rule}' but evaluate_shipped returned None — \
                 the dispatch table is missing an arm for this rule"
            );
        }
    }

    /// Invariant: every name in SHIPPED_RULES must have an explanation string.
    ///
    /// Guards against a rule being registered without a companion .md file
    /// (the explain() function would return None if the include_str! is missing).
    #[test]
    fn all_shipped_rules_have_explanation() {
        for rule in SHIPPED_RULES {
            assert!(
                explain(rule).is_some(),
                "SHIPPED_RULES contains '{rule}' but explain() returned None — \
                 the rule is missing a pub const EXPLANATION or an explain() arm"
            );
        }
    }
}

#[cfg(test)]
mod source_key_tests {
    use serde_json::json;

    use super::source_key;

    #[test]
    fn host_same_with_and_without_port() {
        let a = source_key(&json!("https://example.com/page"));
        let b = source_key(&json!("https://example.com:8080/page"));
        assert_eq!(a, b, "port should be stripped when computing source key");
    }

    #[test]
    fn different_hosts_are_different_sources() {
        let a = source_key(&json!("https://source-a.example.com/page"));
        let b = source_key(&json!("https://source-b.example.com/page"));
        assert_ne!(a, b);
    }

    #[test]
    fn repo_file_uses_path_as_key() {
        let v = json!({"kind": "repo-file", "path": "docs/evidence.md"});
        assert_eq!(source_key(&v), "repo-file:docs/evidence.md");
    }
}
