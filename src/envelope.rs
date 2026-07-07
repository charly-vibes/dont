use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

static CURRENT_AUTHOR: OnceLock<String> = OnceLock::new();

pub fn set_author(author: String) {
    let _ = CURRENT_AUTHOR.set(author);
}

fn current_author() -> Option<String> {
    CURRENT_AUTHOR.get().cloned()
}

pub const ENVELOPE_VERSION: &str = "0.2";
pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvelopeKind {
    Claim,
    Claims,
    Term,
    TermList,
    All,
    Events,
    Rule,
    RuleList,
    RuleResult,
    EvidenceCheck,
    Prime,
    Why,
    Doctor,
    Version,
    Empty,
    Error,
    #[serde(rename = "dont-explain")]
    DontExplain,
    #[serde(rename = "dont-completions")]
    DontCompletions,
    Stats,
    EvalExport,
    Check,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warning {
    pub rule_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationEntry {
    pub command: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnmetClause {
    pub clause: String,
    pub fix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub duration_ms: u64,
    pub tx: Option<u64>,
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResult {
    pub code: String,
    pub message: String,
    pub rule_name: Option<String>,
    pub spec_ref: Option<String>,
    pub entity_id: Option<String>,
    pub unmet_clauses: Vec<UnmetClause>,
    pub remediation: Vec<RemediationEntry>,
}

impl ErrorResult {
    pub fn new(
        code: &str,
        message: &str,
        rule_name: Option<&str>,
        spec_ref: Option<&str>,
        entity_id: Option<&str>,
        unmet_clauses: Vec<UnmetClause>,
        remediation: Vec<RemediationEntry>,
    ) -> Result<Self, &'static str> {
        if remediation.is_empty() {
            return Err("remediation must be non-empty (Invariant 3.2.5)");
        }
        Ok(Self {
            code: code.to_string(),
            message: message.to_string(),
            rule_name: rule_name.map(str::to_string),
            spec_ref: spec_ref.map(str::to_string),
            entity_id: entity_id.map(str::to_string),
            unmet_clauses,
            remediation,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T: Serialize> {
    pub ok: bool,
    pub envelope_version: String,
    pub cli_version: String,
    pub envelope_kind: EnvelopeKind,
    pub data: T,
    pub warnings: Vec<Warning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<Vec<HintEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ephemeral: Option<bool>,
    pub meta: Meta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HintEntry {
    pub command: String,
    pub description: String,
}

impl<T: Serialize> Envelope<T> {
    pub fn success(
        kind: EnvelopeKind,
        data: T,
        warnings: Vec<Warning>,
        hints: Vec<HintEntry>,
    ) -> Self {
        Self {
            ok: true,
            envelope_version: ENVELOPE_VERSION.to_string(),
            cli_version: CLI_VERSION.to_string(),
            envelope_kind: kind,
            data,
            warnings,
            hints: Some(hints),
            ephemeral: None,
            meta: Meta {
                duration_ms: 0,
                tx: None,
                request_id: None,
                author: current_author(),
            },
        }
    }

    pub fn success_with_tx(
        kind: EnvelopeKind,
        data: T,
        warnings: Vec<Warning>,
        hints: Vec<HintEntry>,
        tx: Option<u64>,
    ) -> Self {
        let mut env = Self::success(kind, data, warnings, hints);
        env.meta.tx = tx;
        env
    }
}

impl Envelope<ErrorResult> {
    pub fn error(err: ErrorResult, warnings: Vec<Warning>) -> Self {
        Self {
            ok: false,
            envelope_version: ENVELOPE_VERSION.to_string(),
            cli_version: CLI_VERSION.to_string(),
            envelope_kind: EnvelopeKind::Error,
            data: err,
            warnings,
            hints: None,
            ephemeral: None,
            meta: Meta {
                duration_ms: 0,
                tx: None,
                request_id: None,
                author: current_author(),
            },
        }
    }
}
