use std::path::PathBuf;

use serde_json::Value;

use crate::config::RulesConfig;
use crate::store::Store;

#[derive(Debug, Clone)]
pub struct RuleMatch {
    pub entity_id: String,
    pub detail: String,
}

#[derive(Debug)]
pub enum RuleError {
    Io(std::io::Error),
    Compile { rule_name: String, message: String },
}

impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error reading rule: {e}"),
            Self::Compile { rule_name, message } => {
                write!(f, "rule {rule_name:?} failed to compile: {message}")
            }
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

    /// Load `<rule_name>.dl` from the rules directory and evaluate it against `store`.
    ///
    /// Rules MUST be violation queries whose result columns are `[entity_id, detail]`.
    /// An empty result set means the rule is satisfied; non-empty rows are violations.
    ///
    /// Returns `RuleError::Compile` when the script fails to parse or execute.
    pub fn evaluate(&self, store: &Store, rule_name: &str) -> Result<Vec<RuleMatch>, RuleError> {
        let path = self.rules_dir.join(format!("{rule_name}.dl"));
        let script = std::fs::read_to_string(&path).map_err(RuleError::Io)?;
        store
            .run_rule_query(&script)
            .map(rows_to_matches)
            .map_err(|e| RuleError::Compile {
                rule_name: rule_name.to_string(),
                message: e.to_string(),
            })
    }
}

fn rows_to_matches(rows: Vec<Vec<Value>>) -> Vec<RuleMatch> {
    rows.into_iter()
        .filter_map(|row| {
            let entity_id = row.first()?.as_str()?.to_string();
            let detail = row.get(1)?.as_str()?.to_string();
            Some(RuleMatch { entity_id, detail })
        })
        .collect()
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
        let result = store.append_claim("a verifiable claim", &[]).unwrap();
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
}
