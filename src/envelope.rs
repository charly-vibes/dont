//! Structured CLI output envelope.
//!
//! Re-exports shared types from `genesis::envelope` while preserving
//! dont's `envelope_version` ("0.2") contract and dont-specific
//! `EnvelopeKind` variants.
//!
//! ## Contract (envelope_version 0.2)
//!
//! The JSON envelope shape is: `ok`, `envelope_version`, `cli_version`,
//! `envelope_kind`, `data`, `warnings`, `hints`, `ephemeral`, `meta`.
//! `envelope_version` must be `"0.2"` for backward compatibility with
//! consumers that depend on the dont envelope shape.

pub use genesis::envelope::{ErrorResult, HintEntry, Meta, RemediationEntry, UnmetClause, Warning};

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

/// Local author tracking mirroring genesis's private `CURRENT_AUTHOR`.
///
/// Genesis's `current_author()` is private, so we maintain a local copy
/// for constructing [`Meta`] in our wrapper types. If genesis ever makes
/// `current_author()` public, this local copy and the wrapper `set_author`
/// can be removed in favor of a direct re-export.
static CURRENT_AUTHOR: OnceLock<String> = OnceLock::new();

/// Wrap genesis's `set_author` to also set the local copy.
///
/// Both copies are set so that code using `genesis::envelope::set_author`
/// directly (if any) stays in sync.
pub fn set_author(author: String) {
    let _ = CURRENT_AUTHOR.set(author.clone());
    genesis::envelope::set_author(author);
}

fn current_author() -> Option<String> {
    CURRENT_AUTHOR.get().cloned()
}

/// Envelope protocol version — dont maintains "0.2" for backward compatibility.
pub const ENVELOPE_VERSION: &str = "0.2";

/// CLI version, injected at compile time from dont's `Cargo.toml`.
pub const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Discriminator for the kind of data carried in the envelope.
///
/// dont-specific variants preserved for backward compatibility with
/// the existing command surface. Genesis's generic `EnvelopeKind`
/// is not used directly; this local enum is the canonical one.
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

/// The universal CLI output envelope.
///
/// Structurally identical to `genesis::envelope::Envelope` but uses
/// dont's local `EnvelopeKind` enum and version constants.
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
