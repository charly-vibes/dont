use std::collections::HashMap;

use serde::Deserialize;

/// Validation error returned when a config field holds an invalid value.
#[derive(Debug, Clone)]
pub struct ConfigValidationError {
    pub message: String,
}

impl std::fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ConfigValidationError {}

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub project: ProjectConfig,
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub harness: HarnessConfig,
    #[serde(default)]
    pub import: ImportConfig,
    #[serde(default)]
    pub verify_evidence: VerifyEvidenceConfig,
    #[serde(default)]
    pub define: DefineTopConfig,
    #[serde(default)]
    pub trust: TrustConfig,
    #[serde(default)]
    pub rules: RulesConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProjectConfig {
    pub name: Option<String>,
    pub mode: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct OutputConfig {
    pub default_format: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct StorageConfig {
    pub busy_retry_attempts: Option<u32>,
    pub busy_retry_base_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct HarnessConfig {
    #[serde(default = "default_managed_docs")]
    pub managed_docs: Vec<String>,
    pub spawn_timeout_hours: Option<u64>,
}

fn default_managed_docs() -> Vec<String> {
    vec!["AGENTS.md".to_string(), "CLAUDE.md".to_string()]
}

#[derive(Debug, Deserialize, Default)]
pub struct ImportConfig {
    #[serde(flatten)]
    pub adapters: HashMap<String, AdapterConfig>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct AdapterConfig {
    pub enabled: Option<bool>,
    pub endpoint: Option<String>,
    pub base_url: Option<String>,
}

impl AdapterConfig {
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct VerifyEvidenceConfig {
    pub concurrency: Option<u32>,
    pub rate_limit_per_host: Option<f64>,
    pub burst_per_host: Option<u32>,
    pub retry_limit: Option<u32>,
    pub default_timeout_s: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct DefineTopConfig {
    #[serde(default)]
    pub shape: DefineShapeConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct DefineShapeConfig {
    pub check_indefinite: Option<bool>,
    pub check_punctuated: Option<bool>,
    pub check_compound: Option<bool>,
    pub check_sentence: Option<bool>,
    pub compound_markers: Option<Vec<String>>,
}

impl DefineShapeConfig {
    pub fn check_indefinite(&self) -> bool {
        self.check_indefinite.unwrap_or(true)
    }

    pub fn check_punctuated(&self) -> bool {
        self.check_punctuated.unwrap_or(true)
    }

    pub fn check_compound(&self) -> bool {
        // compound_markers = [] also disables compound check
        if matches!(&self.compound_markers, Some(m) if m.is_empty()) {
            return false;
        }
        self.check_compound.unwrap_or(true)
    }

    pub fn check_sentence(&self) -> bool {
        self.check_sentence.unwrap_or(true)
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct TrustConfig {
    #[serde(default)]
    pub hedges: TrustHedgesConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct TrustHedgesConfig {
    #[serde(default)]
    pub patterns: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct RulesConfig {
    #[serde(default)]
    pub warn: Vec<String>,
    #[serde(default)]
    pub strict: Vec<String>,
    #[serde(default)]
    pub term_nonfunctional: TermNonfunctionalConfig,
    #[serde(default)]
    pub rule_claim_structure: RuleClaimStructureConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct RuleClaimStructureConfig {
    #[serde(default)]
    pub enabled: bool,
    pub tag_term_id: Option<String>,
}

impl RuleClaimStructureConfig {
    pub fn enabled_tag(&self) -> Option<&str> {
        if self.enabled {
            self.tag_term_id.as_deref()
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct TermNonfunctionalConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub patterns: Vec<String>,
}

impl TermNonfunctionalConfig {
    pub fn matches_label(&self, label: &str) -> bool {
        if !self.enabled || self.patterns.is_empty() {
            return false;
        }
        let lower = label.to_lowercase();
        self.patterns.iter().any(|p| lower.contains(p.as_str()))
    }
}

impl Config {
    /// Validate all constrained config fields. Returns an error that names
    /// the failing field and the invalid value so callers can surface it
    /// without ambiguity. Validation runs at load time — not deferred to
    /// individual call sites — so invalid values never reach runtime logic.
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        // [project].mode must be one of the two recognised values when set.
        if let Some(mode) = &self.project.mode {
            let valid = ["permissive", "strict"];
            if !valid.contains(&mode.as_str()) {
                return Err(ConfigValidationError {
                    message: format!(
                        "config field `project.mode` has invalid value {:?}; \
                         must be one of: permissive, strict",
                        mode
                    ),
                });
            }
        }

        // [output].default_format must be one of the recognised output formats when set.
        if let Some(fmt) = &self.output.default_format {
            let valid = ["json", "human"];
            if !valid.contains(&fmt.as_str()) {
                return Err(ConfigValidationError {
                    message: format!(
                        "config field `output.default_format` has invalid value {:?}; \
                         must be one of: json, human",
                        fmt
                    ),
                });
            }
        }

        // [verify_evidence].default_timeout_s must be >= 1 when set.
        if let Some(t) = self.verify_evidence.default_timeout_s {
            if t == 0 {
                return Err(ConfigValidationError {
                    message:
                        "config field `verify_evidence.default_timeout_s` must be >= 1; \
                         got 0 — a zero-second timeout makes all evidence checks fail immediately"
                            .to_string(),
                });
            }
        }

        // [verify_evidence].concurrency must be >= 1 when set.
        if let Some(c) = self.verify_evidence.concurrency {
            if c == 0 {
                return Err(ConfigValidationError {
                    message:
                        "config field `verify_evidence.concurrency` must be >= 1; \
                         got 0 — a concurrency of zero would stall all evidence verification"
                            .to_string(),
                });
            }
        }

        // [verify_evidence].burst_per_host must be >= 1 when set.
        if let Some(b) = self.verify_evidence.burst_per_host {
            if b == 0 {
                return Err(ConfigValidationError {
                    message:
                        "config field `verify_evidence.burst_per_host` must be >= 1; \
                         got 0 — a burst size of zero makes rate limiting block all requests"
                            .to_string(),
                });
            }
        }

        // [verify_evidence].rate_limit_per_host must be > 0.0 when set.
        if let Some(r) = self.verify_evidence.rate_limit_per_host {
            if r <= 0.0 {
                return Err(ConfigValidationError {
                    message: format!(
                        "config field `verify_evidence.rate_limit_per_host` must be > 0; \
                         got {r} — a non-positive rate limit makes evidence verification impossible"
                    ),
                });
            }
        }

        Ok(())
    }
}
