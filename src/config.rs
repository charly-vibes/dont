use std::collections::HashMap;
use std::path::{Path, PathBuf};

use genesis::config::{ConfigError, ConfigFile, ConfigValidation};
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub name: Option<String>,
    pub mode: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct OutputConfig {
    pub default_format: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct StorageConfig {
    pub busy_retry_attempts: Option<u32>,
    pub busy_retry_base_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct HarnessConfig {
    #[serde(default = "default_managed_docs")]
    pub managed_docs: Vec<String>,
    #[serde(default)]
    pub managed_skill_packs: Vec<String>,
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct VerifyEvidenceConfig {
    pub concurrency: Option<u32>,
    pub rate_limit_per_host: Option<f64>,
    pub burst_per_host: Option<u32>,
    pub retry_limit: Option<u32>,
    pub default_timeout_s: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DefineTopConfig {
    #[serde(default)]
    pub shape: DefineShapeConfig,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct TrustConfig {
    #[serde(default)]
    pub hedges: TrustHedgesConfig,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct TrustHedgesConfig {
    #[serde(default)]
    pub patterns: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
    ///
    /// This is the domain validator. The `genesis::config::ConfigFile`
    /// `validate()` override delegates here so `ConfigStore::validate_all`
    /// surfaces the same field errors.
    pub fn validate_fields(&self) -> Result<(), ConfigValidationError> {
        // [project].mode is required: it governs permissive-vs-strict enforcement.
        // Absent means the project has no explicit mode, which silently defaults to
        // an undocumented behaviour — reject with a clear fix instruction instead.
        match &self.project.mode {
            None => {
                return Err(ConfigValidationError {
                    message: "Missing required field `project.mode` in .dont/config.toml. \
                         Add `mode = \"permissive\"` or `mode = \"strict\"` under the \
                         [project] section. Example: mode = \"permissive\""
                        .to_string(),
                });
            }
            Some(mode) => {
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
        }

        // [output].default_format is required: it controls how command output is
        // rendered. Absent means every command silently picks an undocumented default.
        match &self.output.default_format {
            None => {
                return Err(ConfigValidationError {
                    message:
                        "Missing required field `output.default_format` in .dont/config.toml. \
                         Add `default_format = \"json\"` or `default_format = \"human\"` under \
                         the [output] section. Example: default_format = \"json\""
                            .to_string(),
                });
            }
            Some(fmt) => {
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
        }

        // [verify_evidence].default_timeout_s must be >= 1 when set.
        if let Some(t) = self.verify_evidence.default_timeout_s
            && t == 0
        {
            return Err(ConfigValidationError {
                message: "config field `verify_evidence.default_timeout_s` must be >= 1; \
                     got 0 — a zero-second timeout makes all evidence checks fail immediately"
                    .to_string(),
            });
        }

        // [verify_evidence].concurrency must be >= 1 when set.
        if let Some(c) = self.verify_evidence.concurrency
            && c == 0
        {
            return Err(ConfigValidationError {
                message: "config field `verify_evidence.concurrency` must be >= 1; \
                     got 0 — a concurrency of zero would stall all evidence verification"
                    .to_string(),
            });
        }

        // [verify_evidence].burst_per_host must be >= 1 when set.
        if let Some(b) = self.verify_evidence.burst_per_host
            && b == 0
        {
            return Err(ConfigValidationError {
                message: "config field `verify_evidence.burst_per_host` must be >= 1; \
                     got 0 — a burst size of zero makes rate limiting block all requests"
                    .to_string(),
            });
        }

        // [verify_evidence].rate_limit_per_host must be > 0.0 when set.
        if let Some(r) = self.verify_evidence.rate_limit_per_host
            && r <= 0.0
        {
            return Err(ConfigValidationError {
                message: format!(
                    "config field `verify_evidence.rate_limit_per_host` must be > 0; \
                     got {r} — a non-positive rate limit makes evidence verification impossible"
                ),
            });
        }

        Ok(())
    }
}

// ── genesis::config::ConfigFile adoption ──────────────────────────────
//
// dont adopts `genesis::config` for shared config I/O: `read`/`parse` come
// from the trait's blanket impl (delegated to genesis), and `validate`
// forwards domain field checks so a `ConfigStore` can surface them across
// the suite. The marker path is `.dont/config.toml` relative to repo root.

impl ConfigFile for Config {
    fn path(repo_root: &Path) -> PathBuf {
        repo_root.join(".dont").join("config.toml")
    }

    fn validate(&self) -> Result<Vec<ConfigValidation>, ConfigError> {
        match self.validate_fields() {
            Ok(()) => Ok(Vec::new()),
            Err(e) => Ok(vec![ConfigValidation::error("config", e.message)]),
        }
    }
}
