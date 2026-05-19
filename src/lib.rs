//! `dont` — epistemic claim-tracking library.
//!
//! This crate provides the core data model, persistence layer, and workflow
//! primitives for the `dont` CLI. The public modules can also be consumed
//! directly as a library when embedding claim-tracking in other tooling.
//!
//! # Architecture
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`model`] | Claim/term lifecycle state-machine and transition functions |
//! | [`envelope`] | Structured JSON output envelope wrapping all command results |
//! | [`store`] | CozoDB-backed persistence for claims, terms, and events |
//! | [`project`] | Project directory layout, config loading, and `init` / `open` |
//! | [`config`] | Deserialised `.dont/config.toml` types and validation |
//! | [`managed_block`] | Read/write helpers for the managed `DONT` block in docs |
//! | [`fs_util`] | Filesystem utilities (restricted-mode file writes) |
//! | [`linkml`] | LinkML schema import adapter |
//! | [`rules`] | Pluggable lint-rule evaluation engine |
//!
//! # Quick example
//!
//! ```rust,no_run
//! use dont::model::{Status, EntityId};
//!
//! // Parse a user-supplied identifier and inspect its variant.
//! let id = EntityId::parse("term:dont:Claim");
//! assert!(matches!(id, EntityId::Term(_)));
//! assert_eq!(id.as_str(), "term:dont:Claim");
//! ```

/// Configuration types for `.dont/config.toml`.
///
/// [`Config`](config::Config) is the root deserialisation target. Call
/// [`Config::validate`](config::Config::validate) after loading to surface
/// invalid field values before any command logic runs.
pub mod config;

/// Structured JSON output envelope used by every `dont` command.
///
/// All command results are wrapped in an [`Envelope`](envelope::Envelope)
/// that carries `ok`, `envelope_kind`, `data`, `warnings`, and `meta`
/// fields so callers can handle success and error uniformly.
pub mod envelope;

/// Low-level filesystem helpers.
///
/// Currently exposes [`write_restricted`](fs_util::write_restricted), which
/// creates or truncates a file with 0o600 permissions on Unix.
pub mod fs_util;

/// LinkML schema import adapter.
///
/// Parses a LinkML YAML schema and converts it into `dont` terms via
/// [`import_schema`](linkml::import_schema).
pub mod linkml;

/// Managed-block helpers for agent documentation files.
///
/// Provides functions to read, compare, and update the
/// `<!-- DONT:START --> … <!-- DONT:END -->` block that `dont` injects into
/// `AGENTS.md` / `CLAUDE.md` and similar documents.
pub mod managed_block;

/// Core data model: entity statuses and lifecycle transitions.
///
/// [`Status`](model::Status) enumerates the five claim/term states.
/// Each free function (`trust`, `flag`, `ignore`, `undoubt`, `lock`,
/// `reopen`) encodes one legal transition and returns a
/// [`TransitionError`](model::TransitionError) for any illegal move.
pub mod model;

/// Project directory layout, `init`, and `open`.
///
/// [`Project::open`](project::Project::open) locates `.dont/` by walking up
/// from the current directory. [`Project::init`](project::Project::init)
/// creates a new project with a minimal `config.toml`, seed vocabulary, and
/// managed documentation.
pub mod project;

/// Pluggable lint-rule evaluation engine.
///
/// Rules inspect claims and terms and emit [`Warning`](envelope::Warning)
/// values that are attached to command envelopes.
pub mod rules;

/// CozoDB-backed persistence layer.
///
/// [`Store`](store::Store) is the single entry point for all database
/// operations: reading and writing claims, terms, atoms, hypotheses, and
/// events. [`StoreError`](store::StoreError) covers all failure modes.
pub mod store;
