use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
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
