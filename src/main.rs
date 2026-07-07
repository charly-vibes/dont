use std::cell::Cell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use dont::config::{DefineShapeConfig, TermNonfunctionalConfig};
use dont::envelope::{
    CLI_VERSION, Envelope, EnvelopeKind, ErrorResult, HintEntry, RemediationEntry, UnmetClause,
    Warning, set_author,
};
use dont::linkml as linkml_adapter;
use dont::model::{
    EntityId, Status, TransitionError, flag as model_flag, ignore as model_ignore,
    lock as model_lock, reopen as model_reopen, trust as model_trust, undoubt as model_undoubt,
};
use dont::project::{Project, ProjectError, ProjectMode};
use dont::rules::{RuleError, shipped_rule_names};
use dont::skill_pack;
use dont::store::{
    AppendResult, ClaimRecord, CurieResolution, EntityResolution, EventRecord, HypothesisRecord,
    Store, StoreError, StoreEvent, StoreEventKind, TermRecord,
};

thread_local! {
    static HUMAN_MODE: Cell<bool> = const { Cell::new(false) };
    static PLAIN_MODE: Cell<bool> = const { Cell::new(false) };
    static FORCE_COLOR_MODE: Cell<bool> = const { Cell::new(false) };
    static QUIET_MODE: Cell<bool> = const { Cell::new(false) };
    static NO_PERSIST_MODE: Cell<bool> = const { Cell::new(false) };
}

fn human_mode() -> bool {
    HUMAN_MODE.with(|m| m.get())
}

fn quiet_mode() -> bool {
    QUIET_MODE.with(|m| m.get())
}

fn no_persist_mode() -> bool {
    NO_PERSIST_MODE.with(|m| m.get())
}

fn color_enabled() -> bool {
    use std::io::IsTerminal;
    if PLAIN_MODE.with(|m| m.get()) {
        return false;
    }
    if FORCE_COLOR_MODE.with(|m| m.get()) {
        return true;
    }
    if std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()) {
        return false;
    }
    if std::env::var("CLICOLOR_FORCE").ok().as_deref() == Some("1") {
        return true;
    }
    std::io::stdout().is_terminal()
}

fn colorize_status(status: &str) -> String {
    if !color_enabled() {
        return status.to_string();
    }
    match status {
        "unverified" => format!("\x1b[33m{status}\x1b[0m"),
        "doubted" => format!("\x1b[31m{status}\x1b[0m"),
        "verified" => format!("\x1b[32m{status}\x1b[0m"),
        "locked" => format!("\x1b[36m{status}\x1b[0m"),
        "ignored" => format!("\x1b[2m{status}\x1b[0m"),
        _ => status.to_string(),
    }
}

#[derive(Debug, Parser)]
#[command(name = "dont")]
#[command(disable_version_flag = true)]
#[command(disable_help_subcommand = true)]
#[command(about = "Epistemic forcing-function CLI for grounded claims")]
#[command(after_help = "Examples:
  dont init                          # initialise a new project
  dont conclude \"the sky is blue\"    # add an unverified claim
  dont flag claim:abc123 -e https://example.com/sky
  dont lock claim:abc123             # preserve a verified claim
  dont list --status unverified      # see all unverified claims
  dont prime                         # session-start orientation

Command groups:
  Claim lifecycle:
    conclude, trust, undoubt, ignore, reopen, forget, lock
  Evidence and review:
    flag, dismiss, ground, show, why, trace, verify-evidence
  Vocabulary and import:
    define, vocab, import
  Lists and project health:
    list, prime, doctor
  Structured workflows:
    atom (define, dismiss)
    hypothesis (add, assess)
    rules (list, show, add, test)
  Agent guidance:
    help, explain, completions")]
struct Cli {
    /// Print version information. Combine with --json for machine-readable output.
    #[arg(long, global = true)]
    version: bool,

    /// Output JSON envelope on stdout.
    #[arg(long, short = 'j', global = true)]
    json: bool,

    /// Output human-readable text instead of JSON (--json takes precedence).
    #[arg(long, global = true)]
    human: bool,

    /// Output human-readable text without ANSI colours (for logging to files).
    #[arg(long, global = true, conflicts_with = "color")]
    plain: bool,

    /// Force ANSI colour output even when stdout is not a TTY or NO_COLOR is set.
    #[arg(long, global = true, conflicts_with_all = ["plain", "no_color"])]
    color: bool,

    /// Disable ANSI colour output for this invocation.
    #[arg(long = "no-color", global = true, conflicts_with_all = ["plain", "color"])]
    no_color: bool,

    /// Suppress confirmatory output; errors and data output are unaffected.
    #[arg(long, short = 'q', global = true)]
    quiet: bool,

    /// Author identifier for this invocation. Overrides $DONT_AUTHOR.
    #[arg(long, short = 'a', global = true)]
    author: Option<String>,

    /// Bypass harness detection; behave as if DONT_DIRECT=1.
    #[arg(long, global = true)]
    direct: bool,

    /// Validate and check but do not write to the store.
    #[arg(long = "no-persist", global = true)]
    no_persist: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Initialize dont state in the current project.
    #[command(after_help = "Examples:
  dont init             # initialise in permissive mode (default)
  dont init --strict    # initialise in strict mode (all gates enforced)")]
    Init {
        /// Start the project in strict mode instead of permissive mode.
        #[arg(long)]
        strict: bool,
    },

    /// Introduce an unverified claim.
    #[command(after_help = "Examples:
  dont conclude \"the sky is blue\"
  dont conclude \"X causes Y\" --depends-on WB:P001 --confidence 0.85")]
    Conclude {
        /// Claim statement text.
        #[arg(value_name = "statement")]
        statement: String,

        /// CURIE of a term this claim depends on. May be repeated.
        #[arg(long)]
        depends_on: Vec<String>,

        /// LLM-authored confidence score (0.0–1.0). Stored as-is; null if omitted.
        #[arg(long)]
        confidence: Option<f64>,
    },

    /// Introduce an unverified coined term. --doc is required. --label must be a singular
    /// indefinite noun phrase starting with 'a' or 'an', e.g. "a velocity".
    #[command(after_help = "Examples:
  dont define WB:P001 --label \"a velocity\" --doc \"Rate of change of position\"
  dont define --label \"a widget\" --doc \"A reusable UI component\"

Notes:
  --doc is required: provide a non-empty prose definition.
  --label must start with 'a' or 'an' followed by a noun phrase, e.g. \"a commit\".")]
    Define {
        /// Term CURIE, e.g. WB:P001.
        #[arg(value_name = "curie")]
        curie: Option<String>,

        /// Prose definition for the term (required).
        #[arg(long)]
        doc: Option<String>,

        /// Singular indefinite noun phrase starting with 'a' or 'an', e.g. "a velocity".
        #[arg(long)]
        label: Option<String>,
    },

    /// Register doubt — marks a claim or term as Doubted (do not trust it).
    #[command(
        long_about = "Register doubt — marks a claim or term as Doubted (untrustworthy).\n\nNaming: 'dont trust' reads as 'do not trust it' — the 'dont' prefix inverts the verb.\nThis is the counterpart of 'undoubt'. For unambiguous positive phrasing use 'dt challenge'.",
        after_help = "Examples:\n  dont trust claim:abc123 --reason \"No primary source cited\"\n  dont trust term:WB:P001 --reason \"Definition is ambiguous\""
    )]
    Trust {
        /// Claim or term identifier (claim:... or term:...).
        #[arg(value_name = "entity-id")]
        id: String,

        /// Reason for doubt (required).
        #[arg(long, short)]
        reason: Option<String>,
    },

    /// Verify a claim or term with evidence. Read as 'dont flag' = 'do not flag it as a concern'. Alias: dismiss.
    #[command(after_help = "Examples:
  dont flag claim:abc123 -e https://example.com/evidence
  dont flag term:WB:P001 --file docs/spec.md --anchor terminology")]
    Flag {
        /// Claim or term identifier.
        #[arg(value_name = "entity-id")]
        id: String,

        /// Evidence URI or reference.
        #[arg(long, short)]
        evidence: Vec<String>,

        /// Repository-relative file path for a structured evidence locator.
        #[arg(long)]
        file: Option<String>,

        /// URL permalink (e.g. GitHub blob URL with commit hash) for evidence outside the project root. Mutually exclusive with --file.
        #[arg(long, conflicts_with = "file")]
        url: Option<String>,

        /// Line span within the file or URL, e.g. "10-18" or "42".
        #[arg(long)]
        lines: Option<String>,

        /// Named anchor within the file or URL.
        #[arg(long)]
        anchor: Option<String>,

        /// Captured excerpt from the referenced source for later audit.
        #[arg(long)]
        excerpt: Option<String>,
    },

    /// Verify a claim or term with evidence. Canonical glossary core-four verb; alias for flag.
    #[command(after_help = "Examples:
  dont dismiss claim:abc123 -e https://example.com/evidence
  dont dismiss term:WB:P001 --file docs/spec.md --anchor terminology")]
    Dismiss {
        /// Claim or term identifier.
        #[arg(value_name = "entity-id")]
        id: String,

        /// Evidence URI or reference.
        #[arg(long, short)]
        evidence: Vec<String>,

        /// Repository-relative file path for a structured evidence locator.
        #[arg(long)]
        file: Option<String>,

        /// URL permalink (e.g. GitHub blob URL with commit hash) for evidence outside the project root. Mutually exclusive with --file.
        #[arg(long, conflicts_with = "file")]
        url: Option<String>,

        /// Line span within the file or URL, e.g. "10-18" or "42".
        #[arg(long)]
        lines: Option<String>,

        /// Named anchor within the file or URL.
        #[arg(long)]
        anchor: Option<String>,

        /// Captured excerpt from the referenced source for later audit.
        #[arg(long)]
        excerpt: Option<String>,
    },

    /// Retract doubt on a doubted entity, returning it to unverified. Use 'reopen' for ignored entities.
    #[command(after_help = "Examples:
  dont undoubt claim:abc123")]
    Undoubt {
        /// Entity identifier (claim:... or term:...).
        #[arg(value_name = "entity-id")]
        id: String,
    },

    /// Permanently preserve a verified claim when the lockable gate is met. Read as 'dont forget' = 'do not forget it'. Alias: lock.
    #[command(after_help = "Examples:
  dont forget claim:abc123")]
    Forget {
        /// Claim identifier.
        #[arg(value_name = "claim-id")]
        id: String,
    },

    /// Permanently preserve a verified claim when the lockable gate is met. Canonical lifecycle verb. `forget` is a legacy alias.
    #[command(after_help = "Examples:
  dont lock claim:abc123")]
    Lock {
        /// Claim identifier.
        #[arg(value_name = "claim-id")]
        id: String,
    },

    /// [dt] Introduce an unverified claim. Positive-framing alias for 'dt record' ≡ 'dont conclude'.
    #[command(hide = true)]
    Record {
        /// Claim statement text.
        #[arg(value_name = "statement")]
        statement: String,
        /// CURIE of a term this claim depends on. May be repeated.
        #[arg(long)]
        depends_on: Vec<String>,
        /// LLM-authored confidence score (0.0–1.0).
        #[arg(long)]
        confidence: Option<f64>,
    },

    /// [dt] Register explicit doubt. Positive-framing alias for 'dt challenge' ≡ 'dont trust'.
    #[command(hide = true)]
    Challenge {
        /// Claim or term identifier (claim:... or term:...).
        #[arg(value_name = "entity-id")]
        id: String,
        /// Reason for doubt (required).
        #[arg(long, short)]
        reason: Option<String>,
    },

    /// Restore an ignored claim or term to unverified status.
    #[command(after_help = "Examples:
  dont reopen claim:abc123
  dont reopen term:WB:P001")]
    Reopen {
        /// Entity identifier (claim:... or term:...).
        #[arg(value_name = "entity-id")]
        id: String,
    },

    /// Move a claim or term to ignored state.
    #[command(after_help = "Examples:
  dont ignore claim:abc123 --reason \"Superseded by newer evidence\"
  dont ignore term:WB:P001 --reason \"Merged into canonical vocabulary\"")]
    Ignore {
        /// Entity identifier (claim:... or term:...).
        #[arg(value_name = "entity-id")]
        id: String,

        /// Substantive reason for ignoring (required; hedge-only reasons are refused).
        #[arg(long, short)]
        reason: Option<String>,
    },

    /// Show a claim or term.
    #[command(after_help = "Examples:
  dont show claim:abc123
  dont show WB:P001 --history")]
    Show {
        /// Claim or term identifier (claim:ID, term:ID, or CURIE like WB:P001).
        #[arg(value_name = "entity-id")]
        id: String,

        /// Include full event history in the output.
        #[arg(long)]
        history: bool,
    },

    /// Explain why a claim or term has its current status.
    #[command(after_help = "Examples:
  dont why claim:abc123
  dont why term:WB:P001")]
    Why {
        /// Claim or term identifier (claim:ID, term:ID, or CURIE like WB:P001).
        #[arg(value_name = "entity-id")]
        id: String,
    },

    /// Check liveness of attached evidence references without changing status.
    #[command(after_help = "Examples:
  dont verify-evidence claim:abc123
  dont verify-evidence term:WB:P001 --timeout-seconds 5")]
    VerifyEvidence {
        /// Entity identifier (claim:... or term:...).
        #[arg(value_name = "entity-id")]
        id: String,

        /// Per-reference timeout in seconds.
        #[arg(long)]
        timeout_seconds: Option<u64>,
    },

    /// Return session-start orientation and project state summary.
    #[command(after_help = "Examples:
  dont prime")]
    Prime,

    /// Report project diagnostics and optionally repair managed docs.
    #[command(after_help = "Examples:
  dont doctor
  dont doctor --fix --strict")]
    Doctor {
        /// Treat warnings as a non-zero exit.
        #[arg(long)]
        strict: bool,

        /// Rewrite stale managed docs in place.
        #[arg(long)]
        fix: bool,
    },

    /// List entities.
    #[command(after_help = "Examples:
  dont list
  dont list --status unverified
  dont list --kind terms --as-of 2026-05-01")]
    List {
        /// Filter entities by status.
        #[arg(long)]
        status: Option<String>,

        /// Filter claims by derived assessment (e.g. stale, compromised-support).
        #[arg(long)]
        derived_assessment: Option<String>,

        /// Choose whether to list claims or terms.
        #[arg(long)]
        kind: Option<String>,

        /// List both claims and terms together.
        #[arg(long)]
        all: bool,

        /// Evaluate entity state at a historical timestamp (ISO 8601 / RFC 3339 or YYYY-MM-DD).
        #[arg(long, value_name = "TIMESTAMP")]
        as_of: Option<String>,
    },

    /// List term entities (equivalent to `list --kind terms`).
    #[command(after_help = "Examples:
  dont vocab                         # list all terms
  dont vocab --status unverified     # list only unverified terms")]
    Vocab {
        /// Filter terms by status.
        #[arg(long)]
        status: Option<String>,

        /// Evaluate term state at a historical timestamp (ISO 8601 / RFC 3339 or YYYY-MM-DD).
        #[arg(long, value_name = "TIMESTAMP")]
        as_of: Option<String>,
    },

    /// Explain the blocker-path for a claim or term.
    #[command(after_help = "Examples:
  dont trace claim:abc123
  dont trace term:WB:P001")]
    Trace {
        /// Entity identifier (claim:... or term:...).
        #[arg(value_name = "entity-id")]
        id: String,
    },

    /// Generate shell completion scripts.
    #[command(after_help = "Examples:
  dont completions bash
  dont completions fish --json")]
    Completions {
        /// Shell to generate completions for (bash, zsh, fish, powershell, elvish).
        #[arg(value_name = "shell")]
        shell: Shell,
    },

    /// Report usage statistics for a time scope.
    #[command(after_help = "Examples:
  dont stats --json
  dont stats --since 2026-06-01T00:00:00Z --json")]
    Stats {
        /// Scope to a specific session identifier.
        #[arg(long)]
        session: Option<String>,
        /// Include only events at or after this RFC 3339 timestamp.
        #[arg(long)]
        since: Option<String>,
        /// Include only events before this RFC 3339 timestamp.
        #[arg(long)]
        until: Option<String>,
    },

    /// Export structured data for eval harnesses.
    #[command(after_help = "Examples:
  dont export --eval --json
  dont export --eval --session <id> --json")]
    Export {
        /// Export eval-harness structured JSON.
        #[arg(long)]
        eval: bool,
        /// Scope to a specific session identifier.
        #[arg(long)]
        session: Option<String>,
        /// Include only events at or after this RFC 3339 timestamp.
        #[arg(long)]
        since: Option<String>,
        /// Include only events before this RFC 3339 timestamp.
        #[arg(long)]
        until: Option<String>,
    },

    /// Atomically ground a claim with its supporting evidence.
    #[command(after_help = "Examples:
  dont ground \"the sky is blue\" -e https://example.com/source
  dont ground \"water boils at 100C\" --file docs/spec.md --lines 10-12
  dont ground \"function returns AST\" --url https://github.com/owner/repo/blob/abc123def/lib.rs --lines 42-56")]
    Ground {
        /// Claim statement text.
        #[arg(value_name = "statement")]
        statement: String,

        /// Evidence URI or reference.
        #[arg(long, short)]
        evidence: Vec<String>,

        /// Repository-relative file path for a structured evidence locator.
        #[arg(long)]
        file: Option<String>,

        /// URL permalink (e.g. GitHub blob URL with commit hash) for evidence outside the project root. Mutually exclusive with --file.
        #[arg(long, conflicts_with = "file")]
        url: Option<String>,

        /// Line span within the file or URL, e.g. "10-18" or "42".
        #[arg(long)]
        lines: Option<String>,

        /// Named anchor within the file or URL.
        #[arg(long)]
        anchor: Option<String>,

        /// Captured excerpt from the referenced source.
        #[arg(long)]
        excerpt: Option<String>,
    },

    /// Manage independently checkable atoms for a claim.
    #[command(after_help = "Examples:
  dont atom define claim:abc123 --text \"Check the primary source\"
  dont atom dismiss claim:abc123 0 -e https://example.com/evidence")]
    Atom {
        #[command(subcommand)]
        action: AtomAction,
    },

    /// Manage competing hypotheses for a claim.
    #[command(after_help = "Examples:
  dont hypothesis add claim:abc123 --text \"Sensor drift explains the anomaly\"
  dont hypothesis assess claim:abc123 0 --supporting \"Calibration log matches\"")]
    Hypothesis {
        #[command(subcommand)]
        action: HypothesisAction,
    },

    /// Import terms from an external ontology adapter.
    #[command(after_help = "Examples:
  dont import obo chebi.obo
  dont import linkml schema.yaml --json")]
    Import {
        /// Adapter name (obo, ols, wikidata, openalex, bioregistry, jsonld, ttl, linkml).
        #[arg(value_name = "adapter")]
        adapter: String,

        /// Adapter-specific arguments.
        #[arg(
            value_name = "arg",
            trailing_var_arg = true,
            allow_hyphen_values = true
        )]
        args: Vec<String>,
    },

    /// Manage and inspect project rules.
    #[command(after_help = "Examples:
  dont rules list
  dont rules show ungrounded")]
    Rules {
        #[command(subcommand)]
        action: RulesAction,
    },

    /// Show the prose explanation for a rule: what it checks, why it matters, and how to satisfy it.
    #[command(after_help = "Examples:
  dont explain ungrounded
  dont explain lockable --json")]
    Explain {
        /// Rule name (e.g. ungrounded, lockable, correlated-error).
        #[arg(value_name = "rule-name")]
        rule: String,
    },

    /// Show agent-addressed help: command reference, first-session tutorial, and how-to guides.
    #[command(after_help = "Examples:
  dont help
  dont help --tutorial
  dont help --howto rule-claims")]
    Help {
        /// Command name to show help for (same output as <cmd> --help).
        #[arg(value_name = "command")]
        command: Option<String>,

        /// Print the first-session tutorial walkthrough.
        #[arg(long, conflicts_with_all = ["howto", "topics"])]
        tutorial: bool,

        /// List the available tutorial and how-to topic names.
        #[arg(long, conflicts_with_all = ["tutorial", "howto"])]
        topics: bool,

        /// Print the goal-oriented how-to guide for the named topic.
        #[arg(long, value_name = "TOPIC", conflicts_with_all = ["tutorial", "topics"])]
        howto: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum AtomAction {
    /// Add an independently checkable atom to a claim.
    Define {
        /// Claim identifier.
        #[arg(value_name = "claim-id")]
        id: String,

        /// Atom text.
        #[arg(long)]
        text: String,
    },

    /// Mark an atom verified with evidence.
    Dismiss {
        /// Claim identifier.
        #[arg(value_name = "claim-id")]
        id: String,

        /// Atom index (0-based).
        #[arg(value_name = "idx")]
        idx: usize,

        /// Evidence URI or reference. May be repeated.
        #[arg(long, short)]
        evidence: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
enum HypothesisAction {
    /// Record a competing hypothesis for a claim.
    Add {
        /// Claim identifier.
        #[arg(value_name = "claim-id")]
        id: String,

        /// Hypothesis text.
        #[arg(long)]
        text: String,
    },

    /// Assess a hypothesis with supporting or refuting evidence.
    Assess {
        /// Claim identifier.
        #[arg(value_name = "claim-id")]
        id: String,

        /// Hypothesis index (0-based).
        #[arg(value_name = "idx")]
        idx: usize,

        /// Evidence supporting this hypothesis. May be repeated.
        #[arg(long)]
        supporting: Vec<String>,

        /// Evidence refuting this hypothesis. May be repeated.
        #[arg(long)]
        refuting: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
enum RulesAction {
    /// List all active rules with name, severity, and source.
    List,

    /// Show details for one rule, including Datalog source for custom rules.
    Show {
        /// Rule name.
        #[arg(value_name = "rule-name")]
        name: String,
    },

    /// Install a project-specific rule from a .dl file.
    Add {
        /// Path to the .dl file.
        #[arg(value_name = "file")]
        file: PathBuf,
        /// Overwrite an existing rule with the same name.
        #[arg(long)]
        force: bool,
    },

    /// Dry-run a rule against the current store without modifying state.
    Test {
        /// Rule name.
        #[arg(value_name = "rule-name")]
        name: String,
    },
}

#[derive(Debug, Clone, Serialize)]
struct RuleInfo {
    name: String,
    severity: &'static str,
    source: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct RuleDetail {
    name: String,
    severity: &'static str,
    source: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    datalog: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct RuleTestResult {
    rule_name: String,
    severity: &'static str,
    matches: Vec<RuleMatchView>,
}

#[derive(Debug, Clone, Serialize)]
struct RuleMatchView {
    entity_id: String,
    detail: String,
}

const DEFAULT_HEDGES: &[&str] = &["i think", "maybe", "not sure", "probably"];

#[derive(Debug, Clone, Serialize)]
struct EvidenceCheckResult {
    uri: String,
    outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct MockEvidenceCheckResult {
    outcome: String,
    detail: Option<String>,
}

const HELP_TUTORIAL: &[&str] = &[
    "# dont -- First-Session Tutorial\n\n",
    "This walkthrough explains the orient, search, coin, conclude, and spawn loop.\n",
    "Use `dont help <cmd>` for quick reference after the first read.\n\n",
    "## 1. Orient\n\n",
    "Run `dont prime --json` at session start.\n\n",
    "## 2. Search before coining\n\n",
    "Before coining a new term run `dont suggest-term \"<rough concept>\"`.\n\n",
    "## 3. Coin a term\n\n",
    "    dont define WB:P001 --label \"a repository commit\" --doc \"A single atomic change...\"\n\n",
    "Supply `--label '<a noun phrase>'` alongside `--doc`.\n\n",
    "## 4. Record a claim\n\n",
    "    dont conclude \"claim text\"\n\n",
    "Core four verbs: conclude, define, trust, dismiss. Lifecycle verbs: lock, reopen, ignore, verify-evidence.\n\n",
    "## 5. Ground a documented fact (fast path)\n\n",
    "    dont ground \"documented fact\" --file README.md --lines 10-18\n\n",
    "## 6. Handle refusals\n\n",
    "When a command is refused, read `data.remediation[0].command`.\n\n",
    "## 7. Spawn requests\n\n",
    "When a refusal contains a spawn_request envelope, invoke the harness sub-agent.\n\n",
    "## 8. Diagnose blockers\n\n",
    "Run `dont trace <entity-id>` to see the causal path to the root blocker.\n\n",
    "## Further reading\n\n",
    "- `dont help <cmd>` -- per-command reference\n",
    "- `dont help --topics` -- list all how-to topics\n",
    "- `dont help --howto harness-integration` -- integrate dont into a new harness\n",
    "- `.dont/AGENTS.md` -- canonical orientation document\n",
];

const HELP_HOWTO_HARNESS_INTEGRATION: &str = concat!(
    "# How-to: Integrate dont into a new harness\n\n",
    "Wire `dont` commands into an existing CI / agent harness so that spawn requests\n",
    "are fulfilled automatically and structured JSON output is routed correctly.\n\n",
    "1. Set DONT_DIRECT=1 (or pass --direct) in the orchestration layer.\n",
    "2. Parse JSON envelopes: check `ok`, then `data`, hints and `remediation`.\n",
    "3. Fulfil spawn requests: when `data.spawn_request` is non-null, launch a sub-agent.\n",
    "4. Surface remediation: on `ok: false`, read `data.remediation[0].command`.\n",
    "5. Run `dont prime --json` at session start to get blocking entities and mode.\n\n",
    "See `.dont/AGENTS.md` for the full orientation document.\n"
);

const HELP_HOWTO_AUTHORING_RULES: &str = concat!(
    "# How-to: Author a project-specific rule\n\n",
    "Add a custom Datalog rule that enforces a project convention.\n\n",
    "1. Create a .dl file, e.g. rules/my-rule.dl.\n",
    "2. Install: dont rules add rules/my-rule.dl\n",
    "3. Test without modifying state: dont rules test my-rule\n",
    "4. Adjust severity in config.toml under [rules] if needed.\n\n",
    "See `dont explain <rule>` for prose explanations of the shipped rules.\n"
);

const HELP_HOWTO_STORE_RECOVERY: &str = concat!(
    "# How-to: Recover a corrupted .dont/ store\n\n",
    "Restore a project whose .dont/ directory is damaged or inconsistent.\n\n",
    "1. Run `dont doctor --json` to identify which checks fail.\n",
    "2. For stale managed docs run `dont doctor --fix`.\n",
    "3. If db.cozo is corrupt, remove the file and run `dont init`.\n",
    "   Note: this loses all claim and term history in that database.\n",
    "4. The seed snapshot is regenerated by `dont init`.\n",
    "5. If config.toml is missing, recreate via `dont init`.\n\n",
    "Run `dont prime --json` afterwards to confirm the project is healthy.\n"
);

const HELP_HOWTO_RULE_CLAIMS: &str = concat!(
    "# How-to: Author a rule claim\n\n",
    "Document a `dont` rule's behavior as a structured claim using the canonical\n",
    "slot-marker template. The `rule-claim-structure` rule validates mandatory slots.\n\n",
    "## Canonical template\n\n",
    "```\n",
    "[INVOCATION] <rule-name> runs as: background lint | opt-in via `dont check --<flag>`\n",
    "[CONFIG]     Enabled by default: yes | no\n",
    "[MODE]       In permissive mode: warn | strict | same as strict | n/a\n",
    "[TRIGGER]    Fires when: <condition>\n",
    "[GUARD]      Silently skips: <inputs>   (omit if no guard)\n",
    "[EVAL]       Evaluation model: stateless demand | event-driven on <event>   (omit if stateless demand)\n",
    "[BOUNDARY]   Does not handle: <edge cases>; defers to <other-rule>   (omit if no boundary)\n",
    "```\n\n",
    "## Slot reference\n\n",
    "| Slot | Marker | Mandatory | Default when omitted |\n",
    "|------|--------|-----------|----------------------|\n",
    "| INVOCATION MODEL | [INVOCATION] | No | background lint, runs with `dont prime` |\n",
    "| TRIGGER CONDITION | [TRIGGER] | Yes | — |\n",
    "| PRECONDITION GUARD | [GUARD] | No | evaluates all inputs; no silent skip |\n",
    "| EVALUATION MODEL | [EVAL] | No | stateless demand-evaluated |\n",
    "| CONFIG (enablement) | [CONFIG] | Yes — one of CONFIG or MODE | — |\n",
    "| MODE (severity) | [MODE] | Yes — one of CONFIG or MODE | — |\n",
    "| BOUNDARY | [BOUNDARY] | No | no explicit boundary with sibling rules |\n\n",
    "## Mandatory slots\n\n",
    "[TRIGGER] and at least one of [CONFIG] or [MODE] are required.\n",
    "Omitting both is a schema violation flagged by `rule-claim-structure`.\n\n",
    "## Optional slots — when they become load-bearing\n\n",
    "- [INVOCATION]: required when the rule is opt-in (background-lint default is wrong for opt-in rules).\n",
    "- [GUARD]: required when the rule silently skips a non-obvious subset of inputs.\n",
    "- [EVAL]: required when evaluation is event-driven or stateful.\n",
    "- [BOUNDARY]: required when scope is defined by exclusion from a sibling rule.\n\n",
    "## Tagging rule claims\n\n",
    "Tag every rule claim with the `rule-claim-type` term UUID in `--depends-on`:\n\n",
    "    dont conclude \"...\" --depends-on term:<uuid-of-rule-claim-type>\n\n",
    "Do not use the bare CURIE `local:rule-claim-type` — that triggers `unresolved-terms`.\n\n",
    "See `.dont/AGENTS.md` for the full rule claim authoring guide.\n"
);

const HOWTO_TOPICS: &[(&str, &str)] = &[
    (
        "harness-integration",
        "Integrate dont into a new agent harness",
    ),
    ("authoring-rules", "Author a project-specific Datalog rule"),
    ("store-recovery", "Recover a corrupted .dont/ store"),
    ("rule-claims", "Author a structured rule-describing claim"),
];

fn howto_content(topic: &str) -> Option<&'static str> {
    match topic {
        "harness-integration" => Some(HELP_HOWTO_HARNESS_INTEGRATION),
        "authoring-rules" => Some(HELP_HOWTO_AUTHORING_RULES),
        "store-recovery" => Some(HELP_HOWTO_STORE_RECOVERY),
        "rule-claims" => Some(HELP_HOWTO_RULE_CLAIMS),
        _ => None,
    }
}

fn contains_hedge(reason: &str, extra: &[String]) -> bool {
    let lower = reason.to_lowercase();
    DEFAULT_HEDGES.iter().any(|h| lower.contains(h))
        || extra.iter().any(|h| lower.contains(h.as_str()))
}

/// Return `true` if a `linkml` binary is reachable on the current `PATH`.
///
/// Searches each directory in `PATH` (from the environment) for a file named
/// `linkml` (or `linkml.exe` on Windows) that is executable.  No subprocess
/// is spawned — this is a pure filesystem probe so it is fast and testable.
fn linkml_is_on_path() -> bool {
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    let name = if cfg!(target_os = "windows") {
        "linkml.exe"
    } else {
        "linkml"
    };
    std::env::split_paths(&path_var).any(|dir| {
        let candidate = dir.join(name);
        candidate.is_file()
    })
}

fn canonical_source_id_for_local_file(
    schema_path: &Path,
    content: &str,
) -> std::io::Result<String> {
    let realpath = schema_path.canonicalize()?;
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let digest = format!("{:x}", hasher.finalize());
    Ok(format!("linkml:file:{}#{digest}", realpath.display()))
}

/// Handle `dont import linkml <schema.yaml>` by delegating to the linkml adapter.
fn handle_linkml_import(args: &[String], project: &Project) {
    let raw_arg = match args.first() {
        Some(p) => p.as_str(),
        None => {
            emit_error_and_exit(
                refusal(
                    "usage",
                    "usage: dont import linkml <schema.yaml>",
                    None,
                    vec![RemediationEntry {
                        command: "dont import linkml <schema.yaml>".to_string(),
                        description: "Provide a LinkML schema file path".to_string(),
                    }],
                ),
                vec![],
                2,
            );
        }
    };

    if std::env::var_os("PATH")
        .as_deref()
        .is_some_and(|p| p.is_empty())
        && !linkml_is_on_path()
    {
        emit_error_and_exit(
            refusal(
                "config-missing",
                "linkml is not on PATH; install the LinkML CLI before importing schemas",
                None,
                vec![RemediationEntry {
                    command: "pip install linkml".to_string(),
                    description: "install the LinkML CLI, then re-run dont import linkml"
                        .to_string(),
                }],
            ),
            vec![],
            1,
        );
    }

    if raw_arg.starts_with("http://") || raw_arg.starts_with("https://") {
        emit_error_and_exit(
            refusal(
                "network-error",
                &format!(
                    "cannot fetch schema from {raw_arg}: network imports are not yet supported — download the schema locally and retry"
                ),
                None,
                vec![RemediationEntry {
                    command: format!("curl -O {raw_arg} && dont import linkml <downloaded-file>"),
                    description: "Download the schema file locally, then retry the import"
                        .to_string(),
                }],
            ),
            vec![],
            1,
        );
    }

    let schema_path = std::path::Path::new(raw_arg);
    let content = match std::fs::read_to_string(schema_path) {
        Ok(s) => s,
        Err(e) => {
            emit_error_and_exit(
                refusal(
                    "io-error",
                    &format!("cannot read schema file {}: {e}", schema_path.display()),
                    None,
                    vec![RemediationEntry {
                        command: "dont import linkml <schema.yaml>".to_string(),
                        description: "Ensure the file exists and is readable".to_string(),
                    }],
                ),
                vec![],
                1,
            );
        }
    };
    let schema_name = schema_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("schema");
    let canonical_source_id = match canonical_source_id_for_local_file(schema_path, &content) {
        Ok(id) => id,
        Err(e) => {
            emit_error_and_exit(
                refusal(
                    "io-error",
                    &format!(
                        "cannot canonicalize schema file {}: {e}",
                        schema_path.display()
                    ),
                    None,
                    vec![RemediationEntry {
                        command: "dont import linkml <schema.yaml>".to_string(),
                        description: "Ensure the schema file path exists and resolves cleanly"
                            .to_string(),
                    }],
                ),
                vec![],
                1,
            );
        }
    };
    match linkml_adapter::import_schema(schema_name, &content) {
        Err(err) => {
            let unmet: Vec<UnmetClause> = err
                .offending
                .iter()
                .map(|o: &String| UnmetClause {
                    clause: o.clone(),
                    fix: "Remove or simplify this construct".to_string(),
                })
                .collect();
            emit_error_and_exit(
                ErrorResult {
                    code: "linkml-unsupported-feature".to_string(),
                    message: format!(
                        "LinkML schema contains unsupported constructs: {}",
                        err.offending.join(", ")
                    ),
                    rule_name: None,
                    spec_ref: None,
                    entity_id: None,
                    unmet_clauses: unmet,
                    remediation: vec![RemediationEntry {
                        command: "dont import linkml <schema.yaml>".to_string(),
                        description: "Simplify the schema to remove unsupported features"
                            .to_string(),
                    }],
                },
                vec![],
                1,
            );
        }
        Ok(result) => {
            let mut stored = 0u32;
            let mut warnings: Vec<Warning> = result
                .warnings
                .iter()
                .map(|w| Warning {
                    rule_name: format!("linkml-approximate-{}", w.feature),
                    entity_id: None,
                    message: w.message.clone(),
                    suggested_remediation: Some(format!(
                        "Review `{}` usage in {} — dont imports this approximately",
                        w.feature, w.source_name
                    )),
                })
                .collect();
            for term in &result.terms {
                match project.store.append_imported_term(
                    &term.curie,
                    &term.definition,
                    Some(&term.label),
                    &canonical_source_id,
                ) {
                    Ok(_) => stored += 1,
                    Err(StoreError::CurieConflict { .. }) => {
                        // Idempotent re-import: imported term already exists, skip silently.
                    }
                    Err(e) => {
                        warnings.push(Warning {
                            rule_name: "linkml-store-warn".to_string(),
                            entity_id: Some(term.curie.clone()),
                            message: format!("could not store imported term {}: {e}", term.curie),
                            suggested_remediation: None,
                        });
                    }
                }
            }
            let payload = json!({
                "adapter": "linkml",
                "schema_name": schema_name,
                "canonical_source_id": canonical_source_id,
                "stored": stored,
            });
            let env = Envelope::success(EnvelopeKind::Empty, payload, warnings, vec![]);
            emit_json(&env);
        }
    }
}

/// Validate --session and --since/--until flags for analytics commands.
///
/// Exits with an error if the time window is inverted or the session is unknown.
fn validate_scope_flags(session: &Option<String>, since: &Option<String>, until: &Option<String>) {
    if let (Some(s), Some(u)) = (since, until)
        && s.as_str() >= u.as_str()
    {
        emit_error_and_exit(
            refusal(
                "invalid-time-window",
                "--since must be before --until",
                None,
                vec![],
            ),
            vec![],
            1,
        );
    }
    if session.is_some() {
        emit_error_and_exit(
            refusal(
                "unknown-session",
                "session scoping is not yet implemented; no session found with that id",
                None,
                vec![],
            ),
            vec![],
            1,
        );
    }
}

/// Return today's midnight UTC as an RFC 3339 string (e.g. "2026-06-17T00:00:00Z").
fn today_midnight_utc() -> String {
    let now = chrono::Utc::now();
    let midnight = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("midnight is always valid")
        .and_utc();
    midnight.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Build the `scope` JSON object for stats/export payloads.
///
/// When `since` is `None`, defaults to today's midnight UTC (not epoch) per spec.
fn build_scope_value(since: Option<&str>, until: Option<&str>, now: &str) -> serde_json::Value {
    let midnight = today_midnight_utc();
    json!({
        "since": since.unwrap_or(midnight.as_str()),
        "until": until.unwrap_or(now),
    })
}

fn cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Return `"dt"` when invoked as the `dt` binary, `"dont"` otherwise.
///
/// Checked against the basename (without extension) of `argv[0]` so the
/// binary can be installed as either a symlink or a separate copy.
fn active_interface() -> &'static str {
    let argv0 = std::env::args().next().unwrap_or_default();
    let basename = std::path::Path::new(&argv0)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("dont");
    if basename == "dt" { "dt" } else { "dont" }
}

/// Build a cross-vocabulary error for when the wrong interface's command is used.
///
/// Error format: "unknown command '<invoked>' for <interface>. Did you mean '<interface> <suggestion>'?"
fn cross_vocab_refusal(invoked: &str, interface: &str, suggestion: &str) -> ErrorResult {
    refusal(
        "unknown-command",
        &format!(
            "unknown command '{invoked}' for {interface}. Did you mean '{interface} {suggestion}'?"
        ),
        None,
        vec![RemediationEntry {
            command: format!("{interface} {suggestion}"),
            description: format!("Use '{suggestion}' when invoking as '{interface}'"),
        }],
    )
}

fn emit_json<T: serde::Serialize>(envelope: &T) {
    if human_mode() {
        let v = serde_json::to_value(envelope).unwrap();
        println!("{}", format_human(&v));
        if let Some(warnings) = v["warnings"].as_array() {
            for w in warnings {
                if let Some(msg) = w["message"].as_str() {
                    eprintln!("warning: {msg}");
                }
            }
        }
    } else if no_persist_mode() {
        // Inject ephemeral: true into every envelope when --no-persist is active.
        let mut v = serde_json::to_value(envelope).unwrap();
        if let Some(obj) = v.as_object_mut() {
            obj.insert("ephemeral".to_string(), Value::Bool(true));
        }
        println!("{}", v);
    } else {
        println!("{}", serde_json::to_string(envelope).unwrap());
    }
}

/// Like `emit_json` but suppresses output in `--quiet` mode.
/// Use for write/mutation commands whose stdout is confirmatory, not data.
fn emit_confirm_json<T: serde::Serialize>(envelope: &T) {
    if quiet_mode() {
        return;
    }
    emit_json(envelope);
}

fn emit_error_no_exit(err: ErrorResult, warnings: Vec<Warning>, code: i32) -> i32 {
    if human_mode() {
        eprintln!("error: {}", err.message);
        for w in &warnings {
            eprintln!("warning: {}", w.message);
        }
        for r in &err.remediation {
            eprintln!("  run: {}", r.command);
        }
    } else {
        let envelope = Envelope::error(err, warnings);
        emit_json(&envelope);
    }
    code
}

fn emit_error_and_exit(err: ErrorResult, warnings: Vec<Warning>, code: i32) -> ! {
    process::exit(emit_error_no_exit(err, warnings, code));
}

fn handle_store_error_code(err: StoreError, entity_id: Option<&str>) -> i32 {
    if let StoreError::DuplicateClaim {
        text_hash: _,
        existing_id,
    } = err
    {
        return emit_error_no_exit(
            refusal(
                "duplicate-refused",
                &format!("claim with equivalent text already exists as {existing_id}"),
                Some(&existing_id),
                vec![RemediationEntry {
                    command: format!("dont show {existing_id}"),
                    description: "Inspect the existing claim".to_string(),
                }],
            ),
            vec![],
            1,
        );
    }

    if let StoreError::CurieConflict { curie, existing_id } = err {
        return emit_error_no_exit(
            refusal(
                "curie-conflict",
                &format!("CURIE {curie} is already defined by {existing_id}"),
                Some(&existing_id),
                vec![
                    RemediationEntry {
                        command: format!("dont show {existing_id}"),
                        description: "Inspect the existing term before redefining it".to_string(),
                    },
                    RemediationEntry {
                        command: format!(
                            "dont define {} --doc \"<definition>\"",
                            suggest_alternative_curie(&curie)
                        ),
                        description: "Use a different CURIE for a distinct term".to_string(),
                    },
                ],
            ),
            vec![],
            1,
        );
    }

    if let StoreError::AmbiguousPrefix { prefix, candidates } = err {
        return emit_error_no_exit(
            refusal(
                "ambiguous-prefix",
                &format!(
                    "prefix {:?} matches {} entities — use a longer prefix or full ID",
                    prefix,
                    candidates.len()
                ),
                entity_id,
                candidates
                    .iter()
                    .map(|id| RemediationEntry {
                        command: format!("dont show {id}"),
                        description: format!("Show {id}"),
                    })
                    .collect(),
            ),
            vec![],
            1,
        );
    }

    let err_result = ErrorResult {
        code: "internal".to_string(),
        message: err.to_string(),
        rule_name: None,
        spec_ref: None,
        entity_id: entity_id.map(str::to_string),
        unmet_clauses: vec![],
        remediation: vec![
            RemediationEntry {
                command: "ls ${DONT_DIR:-.dont}".to_string(),
                description:
                    "Inspect the project state directory for obvious corruption or missing files"
                        .to_string(),
            },
            RemediationEntry {
                command: "https://github.com/charly-vibes/dont/issues".to_string(),
                description: "Report the issue if the project state looks intact".to_string(),
            },
        ],
    };
    emit_error_no_exit(err_result, vec![], 1)
}

fn run_per_entity<F: FnMut(&str) -> i32>(id: String, mut f: F) -> ! {
    if id != "-" {
        process::exit(f(&id));
    }
    use std::io::BufRead;
    let mut max_code = 0i32;
    for line in std::io::stdin().lock().lines() {
        let raw = line.unwrap_or_default();
        let trimmed = raw.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }
        let code = f(&trimmed);
        if code > max_code {
            max_code = code;
        }
    }
    process::exit(max_code);
}

/// Strip ASCII control characters (including ANSI escape sequences) from a
/// string before it is written to terminal output.
///
/// ANSI injection via user-supplied fields (anchor names, file paths, plain-URI
/// evidence strings) would let a malicious value rewrite the visible terminal
/// line or inject fake status indicators.  We remove every byte < 0x20 and
/// DEL (0x7F) so the raw bytes never reach stdout.
fn strip_control_chars(s: &str) -> String {
    s.chars().filter(|c| !c.is_ascii_control()).collect()
}

/// Maximum number of characters displayed for a single evidence entry line in
/// human-readable output.  Values longer than this are truncated with `…`.
/// Only human output is affected — JSON output always contains the full value.
const EVIDENCE_DISPLAY_MAX: usize = 160;

/// Truncate `s` to at most [`EVIDENCE_DISPLAY_MAX`] Unicode scalar values,
/// appending `…` when the string was shortened.  The truncation boundary is
/// always on a char boundary, so the result is always valid UTF-8.
fn truncate_evidence_for_display(s: &str) -> String {
    let mut chars = s.chars();
    let prefix: String = chars.by_ref().take(EVIDENCE_DISPLAY_MAX).collect();
    if chars.next().is_some() {
        // There are more chars beyond EVIDENCE_DISPLAY_MAX — truncate.
        format!("{prefix}…")
    } else {
        prefix
    }
}

fn format_human(v: &Value) -> String {
    let kind = v.get("envelope_kind").and_then(Value::as_str).unwrap_or("");
    let data = &v["data"];
    match kind {
        "empty" => {
            let mode = data["mode"].as_str().unwrap_or("unknown");
            format!("initialized  {mode} mode")
        }
        "claim" => {
            let id = data["id"].as_str().unwrap_or("?");
            let status = data["status"].as_str().unwrap_or("?");
            let statement = data["statement"].as_str().unwrap_or("?");
            let has_tx = v["meta"]["tx"].is_number();
            if has_tx {
                format!("{}  {id}\n  {statement}", colorize_status(status))
            } else {
                format_claim_detail(data)
            }
        }
        "claims" => format_claims_list(data),
        "term" => {
            let id = data["id"].as_str().unwrap_or("?");
            let status = data["status"].as_str().unwrap_or("?");
            let curie = data["curie"].as_str().unwrap_or("?");
            let has_tx = v["meta"]["tx"].is_number();
            if has_tx {
                format!("{}  {id}  {curie}", colorize_status(status))
            } else {
                format_term_detail(data)
            }
        }
        "term_list" => format_terms_list(data),
        "prime" => format_prime(data),
        "events" => format_trace(data),
        "evidence_check" => format_evidence_check(data),
        "why" => format_why(data),
        "all" => format_all(data),
        "stats" => format_stats(data),
        _ => format!("ok  {kind}"),
    }
}

fn format_claims_list(data: &Value) -> String {
    let items = match data["claims"].as_array().or_else(|| data.as_array()) {
        Some(arr) => arr,
        None => return "(no claims)\nTry: dont conclude \"<claim text>\"".to_string(),
    };
    if items.is_empty() {
        return "(no claims)\nTry: dont conclude \"<claim text>\"".to_string();
    }
    items
        .iter()
        .map(|item| {
            let id = item["id"].as_str().unwrap_or("?");
            let status = item["status"].as_str().unwrap_or("?");
            let stmt = item["statement"].as_str().unwrap_or("?");
            let truncated = if stmt.len() > 70 { &stmt[..70] } else { stmt };
            let pad = " ".repeat(12usize.saturating_sub(status.len()));
            format!("{}{}  {id}  {truncated}", colorize_status(status), pad)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_terms_list(data: &Value) -> String {
    let items = match data.as_array() {
        Some(arr) => arr,
        None => {
            return "(no terms)\nTry: dont define proj:TermName --doc \"<definition>\"".to_string();
        }
    };
    if items.is_empty() {
        return "(no terms)\nTry: dont define proj:TermName --doc \"<definition>\"".to_string();
    }
    items
        .iter()
        .map(|item| {
            let id = item["id"].as_str().unwrap_or("?");
            let status = item["status"].as_str().unwrap_or("?");
            let curie = item["curie"].as_str().unwrap_or("?");
            let pad = " ".repeat(12usize.saturating_sub(status.len()));
            format!("{}{}  {id}  {curie}", colorize_status(status), pad)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_claim_detail(data: &Value) -> String {
    let id = data["id"].as_str().unwrap_or("?");
    let status = data["status"].as_str().unwrap_or("?");
    let statement = data["statement"].as_str().unwrap_or("?");
    let created = data["created_at"].as_str().unwrap_or("?");
    let evidence = data["evidence"].as_array();
    let evidence_str = match evidence {
        Some(e) if !e.is_empty() => e
            .iter()
            .map(|ev| {
                if let Some(s) = ev.as_str() {
                    // Strip control chars to prevent ANSI injection from
                    // user-supplied URI strings reaching the terminal, then
                    // truncate very long values so a pasted 10k-char blob does
                    // not flood the terminal.  JSON output is unaffected.
                    let safe = strip_control_chars(s);
                    format!("    {}", truncate_evidence_for_display(&safe))
                } else if ev.get("kind").and_then(Value::as_str) == Some("repo-file") {
                    let path = strip_control_chars(ev["path"].as_str().unwrap_or("?"));
                    let anchor_suffix = ev["anchor"]
                        .as_str()
                        .map(|a| format!("#{}", strip_control_chars(a)))
                        .unwrap_or_default();
                    let display = format!("repo:{path}{anchor_suffix}");
                    format!("    {}", truncate_evidence_for_display(&display))
                } else if ev.get("kind").and_then(Value::as_str) == Some("url-permalink") {
                    let url = strip_control_chars(ev["url"].as_str().unwrap_or("?"));
                    let anchor_suffix = ev["anchor"]
                        .as_str()
                        .map(|a| format!("#{}", strip_control_chars(a)))
                        .unwrap_or_default();
                    let display = format!("url:{url}{anchor_suffix}");
                    format!("    {}", truncate_evidence_for_display(&display))
                } else {
                    let raw = ev.to_string();
                    format!("    {}", truncate_evidence_for_display(&raw))
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => "    (none)".to_string(),
    };
    let depends = data["depends_on"].as_array();
    let colored_status = colorize_status(status);
    let mut out = format!(
        "{id}\n  status:     {colored_status}\n  statement:  {statement}\n  evidence:\n{evidence_str}\n  created:    {created}"
    );
    if let Some(deps) = depends.filter(|d| !d.is_empty()) {
        let dep_list: Vec<&str> = deps.iter().filter_map(Value::as_str).collect();
        out.push_str(&format!("\n  depends_on: {}", dep_list.join(", ")));
    }
    if let Some(atoms) = data["atoms"].as_array().filter(|a| !a.is_empty()) {
        out.push_str("\n  atoms:");
        for atom in atoms {
            let idx = atom["idx"].as_u64().unwrap_or(0);
            let text = atom["text"].as_str().unwrap_or("?");
            let astatus = atom["status"].as_str().unwrap_or("?");
            let colored = colorize_status(astatus);
            out.push_str(&format!("\n    [{idx}] {text}  ({colored})"));
            if let Some(ev) = atom["evidence"].as_array() {
                for e in ev {
                    let uri = e.as_str().unwrap_or("?");
                    out.push_str(&format!("\n        {uri}"));
                }
            }
        }
    }
    if let Some(hyps) = data["hypotheses"].as_array().filter(|h| !h.is_empty()) {
        out.push_str("\n  hypotheses:");
        for hyp in hyps {
            let idx = hyp["idx"].as_u64().unwrap_or(0);
            let text = hyp["text"].as_str().unwrap_or("?");
            out.push_str(&format!("\n    [{idx}] {text}"));
            let supporting = hyp["assessment"]["supporting"].as_array();
            let refuting = hyp["assessment"]["refuting"].as_array();
            let sup_str = supporting
                .map(|v| {
                    v.iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            let ref_str = refuting
                .map(|v| {
                    v.iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            if !sup_str.is_empty() {
                out.push_str(&format!("\n        supporting: {sup_str}"));
            }
            if !ref_str.is_empty() {
                out.push_str(&format!("\n        refuting:   {ref_str}"));
            }
        }
    }
    out
}

fn format_term_detail(data: &Value) -> String {
    let id = data["id"].as_str().unwrap_or("?");
    let status = data["status"].as_str().unwrap_or("?");
    let curie = data["curie"].as_str().unwrap_or("?");
    let definition = data["definition"].as_str().unwrap_or("(none)");
    let colored_status = colorize_status(status);
    let mut out =
        format!("{id}  {curie}\n  status:      {colored_status}\n  definition:  {definition}");
    if let Some(label) = data["label"].as_str().filter(|l| !l.is_empty()) {
        out.push_str(&format!("\n  label:       {label}"));
    }
    out
}

fn format_prime(data: &Value) -> String {
    let mode = data["mode"].as_str().unwrap_or("?");
    let counts = &data["status_counts"];
    let unverified = counts["unverified"].as_u64().unwrap_or(0);
    let doubted = counts["doubted"].as_u64().unwrap_or(0);
    let verified = counts["verified"].as_u64().unwrap_or(0);
    let locked = counts["locked"].as_u64().unwrap_or(0);
    let ignored = counts["ignored"].as_u64().unwrap_or(0);
    let mut out = format!(
        "dont project  {mode} mode\n  unverified: {unverified}  doubted: {doubted}  verified: {verified}  locked: {locked}  ignored: {ignored}"
    );
    if let Some(blocking) = data["blocking"].as_array().filter(|b| !b.is_empty()) {
        out.push_str("\n\nblocking:");
        for item in blocking {
            let id = item["id"].as_str().unwrap_or("?");
            if let Some(stmt) = item["statement"].as_str() {
                let truncated = if stmt.len() > 60 { &stmt[..60] } else { stmt };
                out.push_str(&format!("\n  {id}  [doubted]  {truncated}"));
            } else if let Some(curie) = item["curie"].as_str() {
                out.push_str(&format!("\n  {id}  [doubted]  {curie}"));
            } else {
                out.push_str(&format!("\n  {id}  [doubted]"));
            }
        }
    }
    out
}

fn format_trace(data: &Value) -> String {
    let id = data["entity_id"].as_str().unwrap_or("?");
    let blockers = data["blockers"]
        .as_array()
        .or_else(|| data["blocker_paths"].as_array());
    match blockers {
        Some(p) if p.is_empty() => format!("{id}  no blockers"),
        Some(p) => {
            let mut out = format!("{id} is blocked by:");
            for blocker in p {
                let path_str = blocker["path"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<_>>()
                            .join(" → ")
                    })
                    .unwrap_or_else(|| blocker.to_string());
                out.push_str(&format!("\n  {path_str}"));
            }
            out
        }
        None => format!("{id}  (trace unavailable)"),
    }
}

fn format_evidence_check(data: &Value) -> String {
    let id = data["entity_id"].as_str().unwrap_or("?");
    let status = data["status"].as_str().unwrap_or("?");
    let mut out = format!("{id} ({status})");
    if let Some(results) = data["results"].as_array() {
        for r in results {
            let uri = r["uri"].as_str().unwrap_or("?");
            let outcome = r["outcome"].as_str().unwrap_or("?");
            let check = match outcome {
                "ok" => "ok",
                "unchecked" => "skip",
                _ => "fail",
            };
            out.push_str(&format!("\n  [{check}] {uri}"));
            if let Some(detail) = r["detail"].as_str() {
                out.push_str(&format!(" ({detail})"));
            }
        }
    }
    out
}

fn format_why(data: &Value) -> String {
    let entity = &data["entity"];
    let entity_kind = entity["entity_kind"].as_str().unwrap_or("claim");
    let mut out = if entity_kind == "term" {
        format_term_detail(entity)
    } else {
        format_claim_detail(entity)
    };
    if let Some(history) = data["history"].as_array().filter(|h| !h.is_empty()) {
        out.push_str("\n  history:");
        for ev in history {
            let at = ev["at"].as_str().unwrap_or("?");
            let kind = ev["event_kind"].as_str().unwrap_or("?");
            let reason = ev["reason"].as_str();
            let author = ev["author"].as_str();
            let mut line = format!("\n    {at}  {kind}");
            if let Some(a) = author {
                line.push_str(&format!("  (by {a})"));
            }
            if let Some(r) = reason {
                line.push_str(&format!("  — {r}"));
            }
            out.push_str(&line);
        }
    }
    if let Some(remediation) = data["remediation"].as_array().filter(|r| !r.is_empty()) {
        out.push_str("\n  remediation:");
        for item in remediation {
            if let Some(cmd) = item["command"].as_str() {
                out.push_str(&format!("\n    run: {cmd}"));
            }
            if let Some(desc) = item["description"].as_str() {
                out.push_str(&format!("\n    {desc}"));
            }
        }
    }
    out
}

fn format_all(data: &Value) -> String {
    let claims_data = json!({"claims": data["claims"].clone()});
    let terms_data = data["terms"].clone();
    let claims_out = format_claims_list(&claims_data);
    let terms_out = format_terms_list(&terms_data);
    let has_claims = data["claims"].as_array().is_some_and(|a| !a.is_empty());
    let has_terms = data["terms"].as_array().is_some_and(|a| !a.is_empty());
    match (has_claims, has_terms) {
        (false, false) => "(no claims or terms)\nTry: dont conclude \"<claim text>\"".to_string(),
        (true, false) => claims_out,
        (false, true) => terms_out,
        (true, true) => format!("{claims_out}\n{terms_out}"),
    }
}

fn format_stats(data: &Value) -> String {
    let scope = &data["scope"];
    let since = scope["since"].as_str().unwrap_or("?");
    let until = scope["until"].as_str().unwrap_or("?");
    let rate = data["claim_verification_rate"]
        .as_f64()
        .map(|r| format!("{:.0}%", r * 100.0))
        .unwrap_or_else(|| "n/a".to_string());
    let contradictions = data["caught_contradiction_count"].as_u64().unwrap_or(0);
    let mut out = format!(
        "stats  {since} → {until}\n  verification rate: {rate}  contradictions caught: {contradictions}"
    );
    if let Some(verbs) = data["verb_counts"].as_object().filter(|m| !m.is_empty()) {
        out.push_str("\n  commands:");
        let mut pairs: Vec<_> = verbs.iter().collect();
        pairs.sort_by_key(|(k, _)| k.as_str());
        for (verb, count) in pairs {
            out.push_str(&format!("\n    {verb}: {count}"));
        }
    }
    out
}

fn project_error_to_exit(err: &ProjectError) -> (String, String, i32) {
    match err {
        ProjectError::AlreadyInitialised(path) => (
            "already-initialised".to_string(),
            format!(
                "project already initialised at {} — re-init would overwrite existing state",
                path.display()
            ),
            1,
        ),
        ProjectError::ConfigMissing(msg) => ("config-missing".to_string(), msg.clone(), 1),
        ProjectError::ConfigInvalid(msg) => ("config-invalid".to_string(), msg.clone(), 1),
        ProjectError::LayoutInvalid(_) => ("layout-invalid".to_string(), err.to_string(), 1),
        ProjectError::Store(_) => ("internal".to_string(), err.to_string(), 1),
        ProjectError::Io { .. } => ("internal".to_string(), err.to_string(), 1),
    }
}

fn remediation_for_project_error(err: &ProjectError) -> Vec<RemediationEntry> {
    match err {
        ProjectError::AlreadyInitialised(path) => vec![RemediationEntry {
            command: format!("ls {}", path.display()),
            description: "Inspect the existing .dont/ directory".to_string(),
        }],
        ProjectError::ConfigMissing(_) => vec![RemediationEntry {
            command: "dont init".to_string(),
            description: "Run dont init to initialise the project".to_string(),
        }],
        ProjectError::ConfigInvalid(_) => vec![RemediationEntry {
            command: "dont prime --json".to_string(),
            description: "Fix the invalid field in config.toml, then re-run".to_string(),
        }],
        ProjectError::LayoutInvalid(_) => vec![RemediationEntry {
            command: "dont init".to_string(),
            description: "Run dont init to repair the missing project directories".to_string(),
        }],
        _ => vec![
            RemediationEntry {
                command: "ls ${DONT_DIR:-.dont}".to_string(),
                description:
                    "Inspect the project state directory for obvious corruption or missing files"
                        .to_string(),
            },
            RemediationEntry {
                command: "https://github.com/charly-vibes/dont/issues".to_string(),
                description: "Report the issue if the project state looks intact".to_string(),
            },
        ],
    }
}

fn emit_project_error_and_exit(err: &ProjectError) -> ! {
    let (code, message, exit) = project_error_to_exit(err);
    let remediation = remediation_for_project_error(err);
    let err_result = ErrorResult {
        code,
        message,
        rule_name: None,
        spec_ref: None,
        entity_id: None,
        unmet_clauses: vec![],
        remediation,
    };
    emit_error_and_exit(err_result, vec![], exit);
}

fn open_project_or_exit() -> Project {
    match Project::open(&cwd()) {
        Ok(p) => {
            // Validate config before any command runs; invalid field values
            // are rejected here with a structured error naming the field.
            if let Err(err) = p.load_validated_config() {
                emit_project_error_and_exit(&err);
            }
            p.check_and_record_mode_change();
            p
        }
        Err(err) => emit_project_error_and_exit(&err),
    }
}

fn parse_claim_status_filter(status: &str) -> Option<Status> {
    match status.trim().to_ascii_lowercase().as_str() {
        "unverified" => Some(Status::Unverified),
        "verified" => Some(Status::Verified),
        "doubted" => Some(Status::Doubted),
        "ignored" => Some(Status::Ignored),
        "locked" => Some(Status::Locked),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListKind {
    Claims,
    Terms,
    All,
}

fn parse_list_kind(kind: &str) -> Option<ListKind> {
    match kind.trim().to_ascii_lowercase().as_str() {
        "claims" => Some(ListKind::Claims),
        "terms" => Some(ListKind::Terms),
        _ => None,
    }
}

/// Try to parse `raw` as a valid `--as-of` timestamp.
/// Accepts RFC 3339 / ISO 8601 datetimes *or* bare `YYYY-MM-DD` date strings.
/// Returns `true` when the value is recognisable.
fn is_valid_as_of(raw: &str) -> bool {
    use chrono::{DateTime, NaiveDate};
    DateTime::parse_from_rfc3339(raw).is_ok() || NaiveDate::parse_from_str(raw, "%Y-%m-%d").is_ok()
}

/// Collect all evidence entries from all events in event-tx order.
/// Each entry is either a plain URI string Value or a structured locator Object Value.
fn collect_evidence_from_events(events: &[EventRecord]) -> Vec<Value> {
    let mut all: Vec<(i64, Value)> = events
        .iter()
        .flat_map(|ev| ev.evidence.iter().map(|v| (ev.tx, v.clone())))
        .collect();
    all.sort_by_key(|(tx, _)| *tx);
    all.into_iter().map(|(_, v)| v).collect()
}

fn project_root_from_store(store: &Store) -> PathBuf {
    store
        .path()
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn fingerprint_text(text: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn locator_line_span(locator: &Value) -> Option<(usize, usize)> {
    let start = locator.get("line_start")?.as_u64()? as usize;
    let end = locator.get("line_end")?.as_u64()? as usize;
    Some((start, end))
}

fn current_locator_text(locator: &Value, project_root: &Path) -> Result<String, String> {
    let path = locator
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| "locator is missing path".to_string())?;
    let full_path = project_root.join(path);
    let canonical_root = project_root.canonicalize().map_err(|err| {
        format!(
            "could not resolve project root {}: {err}",
            project_root.display()
        )
    })?;
    let canonical_target = full_path
        .canonicalize()
        .map_err(|err| format!("could not read {}: {err}", full_path.display()))?;
    if !canonical_target.starts_with(&canonical_root) {
        return Err("path escapes project root".to_string());
    }
    let text = std::fs::read_to_string(&canonical_target)
        .map_err(|err| format!("could not read {}: {err}", full_path.display()))?;
    if let Some((start, end)) = locator_line_span(locator) {
        let lines: Vec<&str> = text.lines().collect();
        if start == 0 || end < start || end > lines.len() {
            return Err(format!(
                "line span {start}-{end} is unavailable in current file with {} lines",
                lines.len()
            ));
        }
        Ok(lines[start - 1..end].join("\n"))
    } else {
        Ok(text)
    }
}

fn locator_audit(locator: &Value, project_root: &Path) -> Value {
    match current_locator_text(locator, project_root) {
        Ok(current_text) => {
            if let Some(stored) = locator.get("fingerprint").and_then(Value::as_str) {
                let current = if locator_line_span(locator).is_none() {
                    locator
                        .get("excerpt")
                        .and_then(Value::as_str)
                        .filter(|excerpt| current_text.contains(*excerpt))
                        .map(fingerprint_text)
                        .unwrap_or_else(|| fingerprint_text(&current_text))
                } else {
                    fingerprint_text(&current_text)
                };
                if current == stored {
                    json!({"status": "current"})
                } else {
                    json!({"status": "drifted", "detail": "stored fingerprint does not match current source slice"})
                }
            } else {
                json!({"status": "current"})
            }
        }
        Err(detail) => json!({"status": "unresolved", "detail": detail}),
    }
}

fn project_evidence_entry(entry: &Value, project_root: &Path) -> Value {
    let Some(obj) = entry.as_object() else {
        return entry.clone();
    };
    if obj.get("kind").and_then(Value::as_str) != Some("repo-file") {
        return entry.clone();
    }
    let mut projected = obj.clone();
    projected.insert("audit".to_string(), locator_audit(entry, project_root));
    Value::Object(projected)
}

fn project_evidence(entries: Vec<Value>, project_root: &Path) -> Vec<Value> {
    entries
        .iter()
        .map(|entry| project_evidence_entry(entry, project_root))
        .collect()
}

fn collect_evidence(record: &ClaimRecord) -> Vec<Value> {
    collect_evidence_from_events(&record.events)
}

fn collect_projected_evidence(record: &ClaimRecord, store: &Store) -> Vec<Value> {
    let project_root = project_root_from_store(store);
    project_evidence(collect_evidence(record), &project_root)
}

fn collect_term_evidence(record: &TermRecord) -> Vec<Value> {
    collect_evidence_from_events(&record.events)
}

fn updated_at(record: &ClaimRecord) -> String {
    record
        .events
        .iter()
        .map(|e| &e.created_at)
        .max()
        .cloned()
        .unwrap_or_else(|| record.created_at.clone())
}

fn build_claim_view(record: &ClaimRecord, store: &Store) -> Value {
    let evidence = collect_projected_evidence(record, store);
    let events: Vec<Value> = record
        .events
        .iter()
        .map(|e| {
            json!({
                "kind": format!("{:?}", e.kind).to_lowercase(),
                "tx": e.tx,
                "created_at": e.created_at,
            })
        })
        .collect();
    json!({
        "id": record.id,
        "entity_kind": "claim",
        "statement": record.statement,
        "status": format!("{:?}", record.status).to_lowercase(),
        "derived_assessments": derived_assessments_for_claim(record, store),
        "confidence": record.confidence.map_or(Value::Null, |c| serde_json::json!(c)),
        "provenance": Value::Null,
        "atoms": record.atoms,
        "hypotheses": record.hypotheses,
        "evidence": evidence,
        "depends_on": record.depends_on,
        "events": events,
        "applicable_rules": {
            "lockable": lockable_rule_view(record, store),
        },
        "created_at": record.created_at,
        "updated_at": updated_at(record),
    })
}

fn build_claim_show_view(record: &ClaimRecord, store: &Store, history: bool) -> Value {
    let mut obj = build_claim_view(record, store);
    if !history {
        obj.as_object_mut().unwrap().remove("events");
    }
    obj
}

fn build_term_view(record: &TermRecord, store: &Store) -> Value {
    let project_root = project_root_from_store(store);
    let evidence = project_evidence(collect_term_evidence(record), &project_root);
    // Note: TermView intentionally omits `updated_at` per spec — term status
    // transitions are tracked through the event history (see `dont why`).
    json!({
        "id": record.id,
        "entity_kind": "term",
        "curie": record.curie,
        "label": record.label,
        "definition": record.definition,
        "kind_of": [],
        "related_to": [],
        "status": format!("{:?}", record.status).to_lowercase(),
        "derived_assessments": [],
        "confidence": Value::Null,
        "provenance": Value::Null,
        "evidence": evidence,
        "created_at": record.created_at,
        "applicable_rules": {},
    })
}

fn build_event_history(events: &[EventRecord]) -> Vec<Value> {
    events
        .iter()
        .map(|event| {
            json!({
                "entity_id": Value::Null,
                "tx": event.tx,
                "event_kind": format!("{:?}", event.kind).to_lowercase(),
                "at": event.created_at,
                "author": Value::Null,
                "reason": event.note,
                "evidence_uri": Value::Null,
                "spawn_request_id": Value::Null,
            })
        })
        .collect()
}

fn build_claim_why_view(record: &ClaimRecord, store: &Store) -> Value {
    let entity = build_claim_view(record, store);
    let unmet = lockable_unmet_clauses(record, store);
    let remediation: Vec<Value> = if unmet.is_empty() {
        vec![]
    } else {
        vec![json!({
            "rule_name": "lockable",
            "command": format!("dont check --lock-readiness {}", record.id),
            "description": unmet.iter().map(|c| c.fix.as_str()).collect::<Vec<_>>().join("; "),
        })]
    };
    json!({
        "entity": entity,
        "history": build_event_history(&record.events),
        "applicable_rules": entity["applicable_rules"].clone(),
        "remediation": remediation,
    })
}

fn build_term_why_view(record: &TermRecord, store: &Store) -> Value {
    let entity = build_term_view(record, store);
    json!({
        "entity": entity,
        "history": build_event_history(&record.events),
        "applicable_rules": entity["applicable_rules"].clone(),
        "remediation": [],
    })
}

fn assessed_hypothesis_count(hypotheses: &[HypothesisRecord]) -> usize {
    hypotheses
        .iter()
        .filter(|h| !h.assessment.supporting.is_empty() || !h.assessment.refuting.is_empty())
        .count()
}

fn evidence_source_key(uri: &str) -> String {
    let without_scheme = uri.split_once("://").map(|(_, rest)| rest).unwrap_or(uri);
    let host = without_scheme
        .split(&['/', '?', '#'][..])
        .next()
        .unwrap_or(without_scheme)
        .trim();
    if host.is_empty() {
        uri.to_string()
    } else {
        host.to_lowercase()
    }
}

fn evidence_entry_source_key(v: &Value) -> String {
    if let Some(uri) = v.as_str() {
        return evidence_source_key(uri);
    }
    if let Some(path) = v
        .as_object()
        .filter(|obj| obj.get("kind").and_then(Value::as_str) == Some("repo-file"))
        .and_then(|obj| obj.get("path"))
        .and_then(Value::as_str)
    {
        return format!("repo-file:{path}");
    }
    if let Some(url) = v
        .as_object()
        .filter(|obj| obj.get("kind").and_then(Value::as_str) == Some("url-permalink"))
        .and_then(|obj| obj.get("url"))
        .and_then(Value::as_str)
    {
        return evidence_source_key(url);
    }
    v.to_string()
}

fn independent_evidence_count(record: &ClaimRecord) -> usize {
    let mut sources = std::collections::BTreeSet::new();
    for entry in collect_evidence(record) {
        sources.insert(evidence_entry_source_key(&entry));
    }
    sources.len()
}

/// Normalize a repository-relative path against `project_root`.
/// Returns the normalized relative path, or an error string describing the violation.
fn normalize_repo_path(
    rel_path: &str,
    project_root: &std::path::Path,
) -> Result<PathBuf, &'static str> {
    use std::path::Component;
    let p = PathBuf::from(rel_path);
    if p.is_absolute() {
        return Err(
            "absolute paths are not allowed as repository locators; use a project-relative path",
        );
    }
    // Walk components to detect escape via `..`
    let mut depth: i64 = 0;
    for component in p.components() {
        match component {
            Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return Err("path escapes project root");
                }
            }
            Component::Normal(_) => depth += 1,
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => {
                return Err("absolute paths are not allowed as repository locators");
            }
        }
    }
    // Build the full path and strip back to relative
    let mut full = project_root.to_path_buf();
    for component in p.components() {
        match component {
            Component::ParentDir => {
                full.pop();
            }
            Component::Normal(c) => full.push(c),
            Component::CurDir => {}
            _ => {}
        }
    }
    if let (Ok(canonical_root), Ok(canonical_target)) =
        (project_root.canonicalize(), full.canonicalize())
        && !canonical_target.starts_with(&canonical_root)
    {
        return Err("path escapes project root");
    }
    Ok(full
        .strip_prefix(project_root)
        .unwrap_or(&full)
        .to_path_buf())
}

/// Validate an evidence URI string supplied via `--evidence`.
///
/// Accepted schemes are `http://` and `https://`. Any other value — including
/// bare strings, `file:` URIs, or strings with unsupported schemes — is
/// rejected with an error that quotes the offending locator.
fn validate_evidence_uri(uri: &str) -> Result<(), String> {
    if uri.starts_with("http://") || uri.starts_with("https://") {
        return Ok(());
    }
    Err(format!(
        "malformed evidence locator \"{uri}\": must be an http:// or https:// URI"
    ))
}

/// Parse a line span string like "10-18" or "42" into (start, end).
fn parse_line_span(s: &str) -> Result<(u32, u32), String> {
    if let Some((a, b)) = s.split_once('-') {
        let start: u32 = a
            .trim()
            .parse()
            .map_err(|_| format!("invalid line span: {s}"))?;
        let end: u32 = b
            .trim()
            .parse()
            .map_err(|_| format!("invalid line span: {s}"))?;
        if start == 0 || end == 0 {
            return Err("line spans are one-based; line 0 is invalid".to_string());
        }
        if start > end {
            return Err(format!("line span start {start} is greater than end {end}"));
        }
        Ok((start, end))
    } else {
        let line: u32 = s
            .trim()
            .parse()
            .map_err(|_| format!("invalid line number: {s}"))?;
        if line == 0 {
            return Err("line spans are one-based; line 0 is invalid".to_string());
        }
        Ok((line, line))
    }
}

/// Build a structured repo-file locator as a JSON Value for storage in the evidence field.
fn build_repo_locator(
    path: &std::path::Path,
    line_span: Option<(u32, u32)>,
    anchor: Option<&str>,
    excerpt: Option<&str>,
) -> Value {
    let mut obj = serde_json::Map::new();
    obj.insert("kind".to_string(), Value::String("repo-file".to_string()));
    obj.insert(
        "path".to_string(),
        Value::String(path.to_string_lossy().into_owned()),
    );
    if let Some((start, end)) = line_span {
        obj.insert("line_start".to_string(), Value::Number(start.into()));
        obj.insert("line_end".to_string(), Value::Number(end.into()));
    }
    if let Some(a) = anchor {
        obj.insert("anchor".to_string(), Value::String(a.to_string()));
    }
    if let Some(e) = excerpt {
        obj.insert("excerpt".to_string(), Value::String(e.to_string()));
    }
    Value::Object(obj)
}

/// Compute derived assessments (stale, compromised-support, etc.) for a claim.
///
/// Note: a claim depending on a Locked term cannot be verified — Locked terms emit
/// "compromised-support" and there is no valid transition from Locked back to a
/// verifiable state without reopening the term first. This is intentional.
fn derived_assessments_for_claim(record: &ClaimRecord, store: &Store) -> Vec<String> {
    let mut derived = Vec::new();
    // Spec: ignored entities always have empty derived_assessments.
    if record.status == Status::Ignored {
        return derived;
    }
    if record.depends_on.is_empty() {
        return derived;
    }

    for dep in &record.depends_on {
        let lookup = if dep.starts_with("term:") {
            store
                .term_by_id(dep)
                .map(|opt| opt.map(CurieResolution::Coined))
        } else {
            store.resolve_curie_reference(dep)
        };
        match lookup {
            Ok(Some(CurieResolution::Coined(term))) => match term.status {
                Status::Verified => {}
                Status::Ignored | Status::Locked => {
                    if !derived.iter().any(|d| d == "compromised-support") {
                        derived.push("compromised-support".to_string());
                    }
                }
                Status::Unverified | Status::Doubted => {
                    if !derived.iter().any(|d| d == "stale") {
                        derived.push("stale".to_string());
                    }
                }
            },
            Ok(Some(CurieResolution::Imported(_))) => {}
            Ok(None) => {
                if !derived.iter().any(|d| d == "unresolved-term") {
                    derived.push("unresolved-term".to_string());
                }
            }
            Err(_) => {
                if !derived.iter().any(|d| d == "dangling-dependency") {
                    derived.push("dangling-dependency".to_string());
                }
            }
        }
    }

    derived
}

#[derive(Debug)]
struct BlockerPath {
    kind: String,
    start_entity: String,
    path: Vec<String>,
    blocking_node: String,
    unresolved_reference: Option<String>,
    remediation: Vec<RemediationEntry>,
}

fn blocker_path_for_dep(
    start_id: &str,
    dep: &str,
    term_result: Result<Option<CurieResolution>, StoreError>,
) -> Option<BlockerPath> {
    let path = vec![start_id.to_string(), dep.to_string()];
    match term_result {
        Ok(Some(CurieResolution::Coined(term))) => {
            let (kind, remediation) = match term.status {
                Status::Unverified | Status::Doubted => (
                    "stale",
                    vec![RemediationEntry {
                        command: format!("dont dismiss {}", term.id),
                        description: format!("Verify the blocking term {}", term.id),
                    }],
                ),
                Status::Ignored | Status::Locked => (
                    "compromised-support",
                    vec![RemediationEntry {
                        command: format!("dont show {}", term.id),
                        description: format!("Inspect the compromised supporting term {}", term.id),
                    }],
                ),
                Status::Verified => return None,
            };
            Some(BlockerPath {
                kind: kind.to_string(),
                start_entity: start_id.to_string(),
                path: vec![start_id.to_string(), term.id.clone()],
                blocking_node: term.id,
                unresolved_reference: None,
                remediation,
            })
        }
        Ok(Some(CurieResolution::Imported(_))) => None,
        Ok(None) => Some(BlockerPath {
            kind: "unresolved-term".to_string(),
            start_entity: start_id.to_string(),
            path,
            blocking_node: dep.to_string(),
            unresolved_reference: Some(dep.to_string()),
            remediation: vec![RemediationEntry {
                command: format!("dont define {dep} --doc \"<definition>\""),
                description: format!("Define the missing term {dep}"),
            }],
        }),
        Err(_) => Some(BlockerPath {
            kind: "dangling-dependency".to_string(),
            start_entity: start_id.to_string(),
            path,
            blocking_node: dep.to_string(),
            unresolved_reference: Some(dep.to_string()),
            remediation: vec![RemediationEntry {
                command: "dont list --kind=term".to_string(),
                description: "List terms to diagnose the missing dependency".to_string(),
            }],
        }),
    }
}

fn trace_claim(record: &ClaimRecord) -> Vec<BlockerPath> {
    use std::collections::HashSet;
    let mut paths: Vec<BlockerPath> = Vec::new();
    // visited prevents duplicate blocker entries and guards against future cyclic deps
    let mut visited: HashSet<String> = HashSet::new();
    visited.insert(record.id.clone());

    let project = match Project::open(&cwd()) {
        Ok(p) => p,
        Err(_) => return paths,
    };

    for dep in &record.depends_on {
        if visited.contains(dep) {
            continue;
        }
        visited.insert(dep.clone());

        let result = if dep.starts_with("term:") {
            project
                .store
                .term_by_id(dep)
                .map(|opt| opt.map(CurieResolution::Coined))
        } else {
            project.store.resolve_curie_reference(dep)
        };

        if let Some(bp) = blocker_path_for_dep(&record.id, dep, result) {
            paths.push(bp);
        }
    }

    paths
}

fn blocker_path_to_value(bp: BlockerPath) -> Value {
    let mut value = json!({
        "kind": bp.kind,
        "start_entity": bp.start_entity,
        "path": bp.path,
        "blocking_node": bp.blocking_node,
        "remediation": bp.remediation.iter().map(|r| json!({
            "command": r.command,
            "description": r.description,
        })).collect::<Vec<_>>(),
    });
    if let Some(reference) = bp.unresolved_reference {
        value["unresolved_reference"] = json!(reference);
    }
    value
}

fn lockable_unmet_clauses(record: &ClaimRecord, store: &Store) -> Vec<UnmetClause> {
    let mut unmet = Vec::new();

    if !matches!(record.status, Status::Verified) {
        unmet.push(UnmetClause {
            clause: format!(
                "claim must be in verified status before locking; has {}",
                record.status.as_str()
            ),
            fix: format!(
                "verify the claim first: dont dismiss {} --evidence <uri>",
                record.id
            ),
        });
    }

    let hypothesis_count = assessed_hypothesis_count(&record.hypotheses);
    if hypothesis_count < 3 {
        unmet.push(UnmetClause {
            clause: format!("needs >=3 assessed hypotheses; has {hypothesis_count}"),
            fix: "record and assess at least three competing hypotheses before locking".to_string(),
        });
    }

    let evidence_count = independent_evidence_count(record);
    if evidence_count < 2 {
        unmet.push(UnmetClause {
            clause: format!(
                "needs >=2 independent supporting evidence items; has {evidence_count}"
            ),
            fix: "attach evidence from at least two independent sources before locking".to_string(),
        });
    }

    for assessment in derived_assessments_for_claim(record, store) {
        unmet.push(UnmetClause {
            clause: format!("derived assessment {assessment} blocks locking"),
            fix: "resolve dependency integrity issues before locking".to_string(),
        });
    }

    unmet
}

fn lockable_rule_view(record: &ClaimRecord, store: &Store) -> Value {
    let unmet: Vec<String> = lockable_unmet_clauses(record, store)
        .into_iter()
        .map(|clause| clause.clause)
        .collect();
    json!({
        "rule_kind": "gate",
        "met": unmet.is_empty(),
        "unmet": unmet,
    })
}

fn dependency_gate_unmet_clauses(record: &ClaimRecord, store: &Store) -> Vec<UnmetClause> {
    derived_assessments_for_claim(record, store)
        .into_iter()
        .map(|assessment| UnmetClause {
            clause: format!("derived assessment {assessment} blocks verification"),
            fix: "resolve dependency integrity issues before dismissing this claim".to_string(),
        })
        .collect()
}

fn dependency_gate_rule_name(unmet_clauses: &[UnmetClause]) -> &'static str {
    if unmet_clauses
        .iter()
        .any(|clause| clause.clause.contains("unresolved-term"))
    {
        "unresolved-terms"
    } else {
        "stale-cascade"
    }
}

fn evidence_check_warning(entity_id: &str, result: &EvidenceCheckResult) -> Option<Warning> {
    let (rule_name, default_detail, remediation) = match result.outcome.as_str() {
        "timeout" => (
            "evidence-timeout",
            "evidence check timed out",
            "Retry with a larger --timeout-seconds value or re-check the cited host later",
        ),
        "malformed" => (
            "evidence-malformed",
            "evidence reference is malformed",
            "Replace the malformed evidence reference with a valid URI",
        ),
        "unreachable" => (
            "evidence-unreachable",
            "evidence reference could not be reached",
            "Confirm the cited host is available or replace the evidence reference",
        ),
        _ => return None,
    };
    Some(Warning {
        rule_name: rule_name.to_string(),
        entity_id: Some(entity_id.to_string()),
        message: format!(
            "{}: {}",
            result.uri,
            result.detail.as_deref().unwrap_or(default_detail)
        ),
        suggested_remediation: Some(remediation.to_string()),
    })
}

fn mocked_evidence_outcomes() -> Option<HashMap<String, MockEvidenceCheckResult>> {
    let raw = std::env::var("DONT_VERIFY_EVIDENCE_MOCK").ok()?;
    serde_json::from_str(&raw).ok()
}

fn check_evidence_uri(
    uri: &str,
    mocks: Option<&HashMap<String, MockEvidenceCheckResult>>,
    _timeout_seconds: Option<u64>,
) -> EvidenceCheckResult {
    if let Some(mock) = mocks.and_then(|m| m.get(uri)) {
        return EvidenceCheckResult {
            uri: uri.to_string(),
            outcome: mock.outcome.clone(),
            detail: mock.detail.clone(),
        };
    }

    if !(uri.starts_with("http://") || uri.starts_with("https://")) {
        return EvidenceCheckResult {
            uri: uri.to_string(),
            outcome: "malformed".to_string(),
            detail: Some("missing URI scheme".to_string()),
        };
    }

    EvidenceCheckResult {
        uri: uri.to_string(),
        outcome: "unchecked".to_string(),
        detail: Some("live HTTP reachability check not yet implemented".to_string()),
    }
}

/// Check git status of a repo-relative path and return `Some("git:<sha>")` if the file is
/// committed and clean, `None` if the project is not inside a git repo, or call
/// `emit_error_and_exit` if the file is untracked, staged, or dirty.
fn check_git_provenance(
    rel_path: &std::path::Path,
    project_root: &std::path::Path,
    entity_id: Option<&str>,
    cmd_prefix: &str,
) -> Option<String> {
    let root = project_root.to_string_lossy();
    let rel = rel_path.to_string_lossy();

    // Unset GIT_DIR, GIT_INDEX_FILE, and GIT_WORK_TREE so that hook environments
    // (e.g. prek/lefthook) do not override the repo that -C discovers from
    // the project_root path.  GIT_WORK_TREE without GIT_DIR is also fatal to git.
    let status = std::process::Command::new("git")
        .args(["-C", &root, "status", "--porcelain", &rel])
        .env_remove("GIT_DIR")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_WORK_TREE")
        .output();

    let output = match status {
        Ok(o) if o.status.success() => o,
        _ => return None, // not a git repo or git unavailable
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next().unwrap_or("");

    if line.is_empty() {
        // HEAD:<path> requires a path relative to the git root (not project_root),
        // and must use forward slashes even on Windows.
        let toplevel_out = std::process::Command::new("git")
            .args(["-C", &root, "rev-parse", "--show-toplevel"])
            .env_remove("GIT_DIR")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_WORK_TREE")
            .output();
        let git_root = match toplevel_out {
            Ok(o) if o.status.success() => {
                std::path::PathBuf::from(String::from_utf8_lossy(&o.stdout).trim().to_string())
            }
            _ => return None,
        };
        let abs_path = project_root.join(rel_path);
        let git_rel = match abs_path.strip_prefix(&git_root) {
            Ok(p) => p.to_string_lossy().replace('\\', "/"),
            Err(_) => return None,
        };
        let sha_out = std::process::Command::new("git")
            .args(["-C", &root, "rev-parse", &format!("HEAD:{git_rel}")])
            .env_remove("GIT_DIR")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_WORK_TREE")
            .output();
        return match sha_out {
            Ok(o) if o.status.success() => {
                let sha = String::from_utf8_lossy(&o.stdout).trim().to_string();
                Some(format!("git:{sha}"))
            }
            _ => None,
        };
    }

    let mut chars = line.chars();
    let index_status = chars.next().unwrap_or(' ');
    let worktree_status = chars.next().unwrap_or(' ');

    if index_status == '?' {
        emit_error_and_exit(
            refusal(
                "untracked-file",
                "file is not tracked by git; commit it before using as evidence",
                entity_id,
                vec![RemediationEntry {
                    command: format!("git add {rel} && git commit"),
                    description: "Track and commit the file first".to_string(),
                }],
            ),
            vec![],
            1,
        );
    }

    if worktree_status != ' ' {
        emit_error_and_exit(
            refusal(
                "dirty-file",
                "file has unstaged modifications; SHA would not match current content",
                entity_id,
                vec![RemediationEntry {
                    command: format!("{cmd_prefix} --file <committed-path>"),
                    description: "Commit the modifications first".to_string(),
                }],
            ),
            vec![],
            1,
        );
    }

    // index_status is non-space, non-? → staged but not committed
    emit_error_and_exit(
        refusal(
            "staged-not-committed",
            "file is staged but not yet committed; no SHA exists to reference",
            entity_id,
            vec![RemediationEntry {
                command: "git commit".to_string(),
                description: "Commit the staged file first".to_string(),
            }],
        ),
        vec![],
        1,
    );
}

/// Validate and resolve a `--file` locator into a `Value` suitable for the evidence array.
/// Calls `emit_error_and_exit` on any validation failure.
fn resolve_file_locator(
    file_path: &str,
    lines: Option<&str>,
    anchor: Option<&str>,
    excerpt: Option<&str>,
    project_root: &std::path::Path,
    entity_id: Option<&str>,
    cmd_prefix: &str,
) -> Value {
    if PathBuf::from(file_path).is_absolute() {
        emit_error_and_exit(
            refusal(
                "path-not-relative",
                "repository evidence locators must be project-relative paths, not absolute",
                entity_id,
                vec![RemediationEntry {
                    command: format!("{cmd_prefix} --file <relative-path>"),
                    description: "Use a path relative to the project root".to_string(),
                }],
            ),
            vec![],
            1,
        );
    }
    let normalized = match normalize_repo_path(file_path, project_root) {
        Ok(p) => p,
        Err(msg) => emit_error_and_exit(
            refusal(
                "path-escapes-root",
                &format!("evidence locator path is invalid: {msg}"),
                entity_id,
                vec![RemediationEntry {
                    command: format!("{cmd_prefix} --file <relative-path>"),
                    description: "Use a path that stays within the project root".to_string(),
                }],
            ),
            vec![],
            1,
        ),
    };
    let commit_ref = check_git_provenance(&normalized, project_root, entity_id, cmd_prefix);
    let line_span = match lines.map(parse_line_span) {
        Some(Ok(span)) => Some(span),
        Some(Err(msg)) => emit_error_and_exit(
            refusal(
                "invalid-line-span",
                &format!("invalid --lines value: {msg}"),
                entity_id,
                vec![RemediationEntry {
                    command: format!("{cmd_prefix} --file {file_path} --lines <start-end>"),
                    description: "Use a format like \"10-18\" or \"42\"".to_string(),
                }],
            ),
            vec![],
            1,
        ),
        None => None,
    };
    let current_text = match current_locator_text(
        &build_repo_locator(&normalized, line_span, anchor, None),
        project_root,
    ) {
        Ok(text) => text,
        Err(msg) => emit_error_and_exit(
            refusal(
                "unreadable-evidence",
                &format!("repository evidence locator could not be read: {msg}"),
                entity_id,
                vec![RemediationEntry {
                    command: format!("{cmd_prefix} --file <existing-relative-path>"),
                    description: "Use an existing file inside the project root".to_string(),
                }],
            ),
            vec![],
            1,
        ),
    };
    let fingerprint_source = if line_span.is_some() {
        current_text.as_str()
    } else {
        excerpt.unwrap_or(&current_text)
    };
    let stored_excerpt = excerpt
        .map(str::to_string)
        .or_else(|| line_span.map(|_| current_text.clone()));
    let mut locator = build_repo_locator(&normalized, line_span, anchor, stored_excerpt.as_deref());
    if let Value::Object(obj) = &mut locator {
        obj.insert(
            "fingerprint".to_string(),
            Value::String(fingerprint_text(fingerprint_source)),
        );
        if let Some(cref) = commit_ref {
            obj.insert("commit_ref".to_string(), Value::String(cref));
        }
    }
    locator
}

/// Build a structured URL-permalink locator as a JSON Value for evidence outside
/// the project root. The URL should be a permanent link (e.g. a GitHub blob URL
/// pinned to a specific commit hash) so the evidence remains reproducible.
fn resolve_url_locator(
    url: &str,
    lines: Option<&str>,
    anchor: Option<&str>,
    excerpt: Option<&str>,
) -> Value {
    let line_span = match lines.map(parse_line_span) {
        Some(Ok(span)) => Some(span),
        Some(Err(msg)) => emit_error_and_exit(
            refusal(
                "invalid-line-span",
                &format!("invalid --lines value: {msg}"),
                None,
                vec![RemediationEntry {
                    command: format!("dont ground --url {url} --lines <start-end>"),
                    description: "Use a format like \"10-18\" or \"42\"".to_string(),
                }],
            ),
            vec![],
            1,
        ),
        None => None,
    };
    let mut obj = serde_json::Map::new();
    obj.insert(
        "kind".to_string(),
        Value::String("url-permalink".to_string()),
    );
    obj.insert("url".to_string(), Value::String(url.to_string()));
    if let Some((start, end)) = line_span {
        obj.insert("line_start".to_string(), Value::Number(start.into()));
        obj.insert("line_end".to_string(), Value::Number(end.into()));
    }
    if let Some(a) = anchor {
        obj.insert("anchor".to_string(), Value::String(a.to_string()));
    }
    if let Some(e) = excerpt {
        obj.insert("excerpt".to_string(), Value::String(e.to_string()));
    }
    Value::Object(obj)
}

/// Build a typed not-found error based on the input form:
/// - `claim:*`  → "claim-not-found"
/// - `term:*`   → "term-not-found"
/// - CURIE (`NS:local`, not claim/term prefix) → "term-not-found" with curie phrasing
/// - bare (no colon) → "entity-not-found"
fn entity_not_found_error(input: &str) -> (&'static str, String, Vec<RemediationEntry>) {
    if input.starts_with("claim:") {
        (
            "claim-not-found",
            format!("no claim with id {input}"),
            vec![RemediationEntry {
                command: "dont list".to_string(),
                description: "List all claims to find the correct id".to_string(),
            }],
        )
    } else if input.starts_with("term:") {
        (
            "term-not-found",
            format!("no term with id {input}"),
            vec![RemediationEntry {
                command: "dont vocab".to_string(),
                description: "List terms to find the correct id".to_string(),
            }],
        )
    } else if input.contains(':') {
        // CURIE
        (
            "term-not-found",
            format!("no term with curie {input}"),
            vec![RemediationEntry {
                command: "dont vocab".to_string(),
                description: "List terms to find the correct curie".to_string(),
            }],
        )
    } else {
        (
            "entity-not-found",
            format!("no entity matching {input:?}"),
            vec![
                RemediationEntry {
                    command: "dont list".to_string(),
                    description: "List all claims".to_string(),
                },
                RemediationEntry {
                    command: "dont vocab".to_string(),
                    description: "List all terms".to_string(),
                },
            ],
        )
    }
}

fn severity_label(s: dont::rules::Severity) -> &'static str {
    match s {
        dont::rules::Severity::Strict => "strict",
        dont::rules::Severity::Warn => "warn",
    }
}

/// Run the `ungrounded` rule and write a rejection event file when `DONT_EMIT_EVENTS=1`.
/// Errors are silently swallowed — event emission is best-effort and must not break doctor.
fn emit_ungrounded_events_if_enabled(project: &dont::project::Project) {
    if std::env::var(dont::events::EVENTS_ENV).is_err() {
        return;
    }
    let rules_dir = project.dont_dir.join("rules");
    let config = project.load_config();
    let engine = dont::rules::RuleEngine::new(
        rules_dir,
        config.rules,
        project.mode() == Some(dont::project::ProjectMode::Strict),
    );
    if let Some(Ok(matches)) = engine.evaluate_shipped(&project.store, "ungrounded") {
        let _ = dont::events::emit_if_enabled(&project.dont_dir, "ungrounded", &matches);
    }
}

fn refusal(
    code: &str,
    message: &str,
    entity_id: Option<&str>,
    remediation: Vec<RemediationEntry>,
) -> ErrorResult {
    ErrorResult {
        code: code.to_string(),
        message: message.to_string(),
        rule_name: None,
        spec_ref: None,
        entity_id: entity_id.map(str::to_string),
        unmet_clauses: vec![],
        remediation,
    }
}

fn handle_store_error(err: StoreError, entity_id: Option<&str>) -> ! {
    process::exit(handle_store_error_code(err, entity_id));
}

fn suggest_alternative_curie(curie: &str) -> String {
    match curie.rsplit_once(':') {
        Some((prefix, local)) => format!("{prefix}:{}_2", local),
        None => format!("{curie}_2"),
    }
}

fn emit_claim_view(record: &ClaimRecord, result: &AppendResult, store: &Store) {
    let payload = build_claim_view(record, store);
    let env = Envelope::success_with_tx(
        EnvelopeKind::Claim,
        payload,
        vec![],
        vec![HintEntry {
            command: format!("dont show {}", record.id),
            description: "Inspect the updated claim".to_string(),
        }],
        Some(result.tx as u64),
    );
    emit_confirm_json(&env);
}

fn emit_term_view(
    record: &TermRecord,
    result: &AppendResult,
    store: &Store,
    warnings: Vec<Warning>,
) {
    let payload = build_term_view(record, store);
    let env = Envelope::success_with_tx(
        EnvelopeKind::Term,
        payload,
        warnings,
        vec![HintEntry {
            command: format!("dont show {}", record.id),
            description: "Inspect the new term".to_string(),
        }],
        Some(result.tx as u64),
    );
    emit_confirm_json(&env);
}

fn transition_not_found_refusal(code: &str, id: &str) -> ErrorResult {
    match code {
        "claim-not-found" => refusal(
            code,
            &format!("no claim with id {id}"),
            Some(id),
            vec![RemediationEntry {
                command: "dont list".to_string(),
                description: "List all claims to find the correct id".to_string(),
            }],
        ),
        "term-not-found" => refusal(
            code,
            &format!("no term with id {id}"),
            Some(id),
            vec![RemediationEntry {
                command: "dont vocab".to_string(),
                description: "List terms to find the correct id".to_string(),
            }],
        ),
        _ => refusal(
            code,
            &format!("no entity with id {id}"),
            Some(id),
            vec![RemediationEntry {
                command: "dont list".to_string(),
                description: "List all entities to find the correct id".to_string(),
            }],
        ),
    }
}

fn transition_invalid_refusal(id: &str, err: &TransitionError) -> ErrorResult {
    refusal(
        &err.code,
        &err.message,
        Some(id),
        vec![RemediationEntry {
            command: format!("dont show {id}"),
            description: "Inspect the current entity status".to_string(),
        }],
    )
}

fn apply_claim_transition_impl(
    project: &Project,
    id: &str,
    transition: fn(Status) -> Result<Status, TransitionError>,
    event: StoreEvent,
    missing_code: &str,
    action: &str,
    allow_evidence_append_on_verified: bool,
) {
    let record = match project.store.claim_by_id(id) {
        Ok(Some(record)) => record,
        Ok(None) => emit_error_and_exit(transition_not_found_refusal(missing_code, id), vec![], 1),
        Err(err) => handle_store_error(err, Some(id)),
    };
    let current = record.status;

    if no_persist_mode() {
        // Validate transition without writing.
        match transition(current) {
            Ok(_new_status) => {}
            Err(_) if allow_evidence_append_on_verified && current == Status::Verified => {}
            Err(err) => emit_error_and_exit(transition_invalid_refusal(id, &err), vec![], 1),
        }
        // Emit the current record as if the transition had occurred (status unchanged).
        let fake_result = dont::store::AppendResult {
            id: id.to_string(),
            event_id: String::new(),
            tx: 0,
            created_at: dont::store::now_rfc3339_pub(),
        };
        emit_claim_view(&record, &fake_result, &project.store);
        return;
    }

    let result = match transition(current) {
        Ok(new_status) => match project
            .store
            .append_status_change(id, current, new_status, event)
        {
            Ok(result) => result,
            Err(err) => handle_store_error(err, Some(id)),
        },
        Err(_) if allow_evidence_append_on_verified && current == Status::Verified => {
            match project.store.append_evidence_event(id, event) {
                Ok(result) => result,
                Err(err) => handle_store_error(err, Some(id)),
            }
        }
        Err(err) => emit_error_and_exit(transition_invalid_refusal(id, &err), vec![], 1),
    };
    let updated = match project.store.claim_by_id(id) {
        Ok(Some(record)) => record,
        Ok(None) => handle_store_error(
            StoreError::Malformed(format!("claim {id} vanished after {action}")),
            Some(id),
        ),
        Err(err) => handle_store_error(err, Some(id)),
    };
    emit_claim_view(&updated, &result, &project.store);
}

fn apply_claim_transition(
    project: &Project,
    id: &str,
    transition: fn(Status) -> Result<Status, TransitionError>,
    event: StoreEvent,
    missing_code: &str,
    action: &str,
) {
    apply_claim_transition_impl(project, id, transition, event, missing_code, action, false);
}

fn apply_claim_transition_or_append_evidence(
    project: &Project,
    id: &str,
    transition: fn(Status) -> Result<Status, TransitionError>,
    event: StoreEvent,
    missing_code: &str,
    action: &str,
) {
    apply_claim_transition_impl(project, id, transition, event, missing_code, action, true);
}

fn apply_term_transition(
    project: &Project,
    id: &str,
    transition: fn(Status) -> Result<Status, TransitionError>,
    event: StoreEvent,
    missing_code: &str,
    _action: &str,
) {
    let record = match project.store.term_by_id(id) {
        Ok(Some(record)) => record,
        Ok(None) => emit_error_and_exit(transition_not_found_refusal(missing_code, id), vec![], 1),
        Err(err) => handle_store_error(err, Some(id)),
    };
    let current = record.status;
    match transition(current) {
        Ok(_) => {}
        Err(err) => emit_error_and_exit(transition_invalid_refusal(id, &err), vec![], 1),
    }
    if no_persist_mode() {
        let fake_result = dont::store::AppendResult {
            id: id.to_string(),
            event_id: String::new(),
            tx: 0,
            created_at: dont::store::now_rfc3339_pub(),
        };
        emit_term_view(&record, &fake_result, &project.store, vec![]);
        return;
    }
    let new_status = transition(current).expect("already validated above");
    let result = match project
        .store
        .append_term_status_change(id, current, new_status, event)
    {
        Ok(result) => result,
        Err(err) => handle_store_error(err, Some(id)),
    };
    let updated = match project.store.term_by_id(id) {
        Ok(Some(record)) => record,
        Ok(None) => handle_store_error(
            StoreError::Malformed(format!("term {id} vanished after update")),
            Some(id),
        ),
        Err(err) => handle_store_error(err, Some(id)),
    };
    emit_term_view(&updated, &result, &project.store, vec![]);
}

// Vowel-letter heuristic only — "uniform" → "an uniform" (wrong). Acceptable because
// call sites pass controlled strings from validated label input, not arbitrary nouns.
fn best_article_for(noun: &str) -> &'static str {
    match noun
        .split_whitespace()
        .next()
        .and_then(|w| w.chars().next())
        .map(|c| c.to_ascii_lowercase())
    {
        Some('a' | 'e' | 'i' | 'o' | 'u') => "an",
        _ => "a",
    }
}

fn label_has_indefinite_article(label: &str) -> bool {
    let mut words = label.split_whitespace();
    matches!(
        words.next().map(|w| w.to_ascii_lowercase()).as_deref(),
        Some("a") | Some("an")
    ) && words.next().is_some()
}

fn label_ends_with_sentence_punctuation(label: &str) -> bool {
    let trimmed = label.trim_end_matches(|c: char| c.is_ascii_whitespace());
    matches!(trimmed.chars().last(), Some('.' | '?' | '!' | ';' | ':'))
}

fn label_compound_undeclared(label: &str) -> bool {
    const MARKERS: &[(&str, Option<usize>)] = &[
        ("a pair", Some(2)),
        ("a triple", Some(3)),
        ("a quadruple", Some(4)),
        ("a sequence", None),
        ("a tuple", None),
        ("a set of", None),
        ("a list of", None),
    ];
    let lower = label.to_ascii_lowercase();
    for &(prefix, required) in MARKERS {
        if !lower.starts_with(prefix) {
            continue;
        }
        let after = &lower[prefix.len()..];
        if !after.is_empty() && !after.starts_with(|c: char| c.is_ascii_whitespace() || c == '(') {
            continue;
        }
        if let Some(open_rel) = label[prefix.len()..].find('(') {
            let open = prefix.len() + open_rel;
            if let Some(close_rel) = label[open..].find(')') {
                let close = open + close_rel;
                let var_count = label[open + 1..close]
                    .split(',')
                    .filter(|s| !s.trim().is_empty())
                    .count();
                return match required {
                    Some(n) => var_count != n,
                    None => var_count == 0,
                };
            }
        }
        return true;
    }
    false
}

fn label_compound_undeclared_with_markers(label: &str, shape: &DefineShapeConfig) -> bool {
    if let Some(markers) = &shape.compound_markers {
        if markers.is_empty() {
            return false;
        }
        let lower = label.to_ascii_lowercase();
        for prefix in markers {
            let prefix_lower = prefix.to_ascii_lowercase();
            if !lower.starts_with(&prefix_lower) {
                continue;
            }
            let after = &lower[prefix_lower.len()..];
            if !after.is_empty()
                && !after.starts_with(|c: char| c.is_ascii_whitespace() || c == '(')
            {
                continue;
            }
            if let Some(open_rel) = label[prefix_lower.len()..].find('(') {
                let open = prefix_lower.len() + open_rel;
                if let Some(close_rel) = label[open..].find(')') {
                    let close = open + close_rel;
                    let var_count = label[open + 1..close]
                        .split(',')
                        .filter(|s| !s.trim().is_empty())
                        .count();
                    if var_count == 0 {
                        return true;
                    }
                    continue;
                }
            }
            return true;
        }
        false
    } else {
        label_compound_undeclared(label)
    }
}

fn words_contain(set: &[&str], text: &str) -> bool {
    text.split_whitespace()
        .any(|w| set.iter().any(|&s| s == w.to_ascii_lowercase()))
}

fn label_contains_sentence_verb(label: &str) -> bool {
    const VERBS: &[&str] = &["is", "are", "has", "have", "does", "do", "was", "were"];
    let parens = label
        .find('(')
        .and_then(|open| label[open..].find(')').map(|rel| (open, open + rel + 1)));
    if let Some((open, close)) = parens {
        if words_contain(VERBS, &label[..open]) {
            return true;
        }
        let after = &label[close..];
        let mut found_verb = false;
        for w in after.split_whitespace() {
            if w.eq_ignore_ascii_case("where") {
                break;
            }
            if VERBS.iter().any(|&v| v == w.to_ascii_lowercase()) {
                found_verb = true;
                break;
            }
        }
        found_verb
    } else {
        words_contain(VERBS, label)
    }
}

/// Reject claim statements that contain path separators or shell metacharacters.
///
/// Claim statements are stored as free-form prose in the database and echoed
/// in JSON output. Although `dont` never interpolates them into shell strings
/// or uses them as filenames today, characters like `../`, `;`, `|`, `` ` ``,
/// and `$` create injection risks if a downstream harness embeds the statement
/// in a shell command, and path-separator sequences (`..`, `/`, `\`) are
/// semantically meaningless in a prose claim.
///
/// Allowlist approach: statements must consist only of printable characters
/// that are not shell metacharacters or path-construction tokens.
fn validate_claim_statement(statement: &str, command: &str) -> Option<ErrorResult> {
    // Characters that are unambiguously dangerous in shell/path contexts.
    const SHELL_META: &[char] = &[';', '|', '`', '$', '\\', '<', '>', '\0'];
    // Path separator sequences: `/` alone is allowed in prose (e.g., "TCP/IP"),
    // but `..` adjacent to a path separator signals traversal. We ban the
    // backslash (already in SHELL_META) and bare NUL.

    if let Some(bad) = statement.chars().find(|c| SHELL_META.contains(c)) {
        return Some(refusal(
            "statement-contains-metacharacter",
            &format!(
                "statement: must not contain shell metacharacters or path separators; \
                 found {:?}; expected printable prose characters only",
                bad
            ),
            None,
            vec![RemediationEntry {
                command: format!("{command} \"<claim text>\""),
                description:
                    "Re-run with a statement that contains only printable prose characters"
                        .to_string(),
            }],
        ));
    }

    // Reject `..` only when adjacent to a path separator (path traversal sequence).
    // This allows valid prose like "versions 1..10" or "pre..post".
    let has_traversal = statement.contains("../")
        || statement.contains("..\\")
        || statement.starts_with("../")
        || statement == "..";
    if has_traversal {
        return Some(refusal(
            "statement-contains-path-traversal",
            "statement: must not contain the path traversal sequence '..'; expected printable prose characters only",
            None,
            vec![RemediationEntry {
                command: format!("{command} \"<claim text>\""),
                description: "Remove the path traversal sequence from the statement".to_string(),
            }],
        ));
    }

    None
}

fn validate_label(label: &str, curie: &str, shape: &DefineShapeConfig) -> Option<ErrorResult> {
    if label.trim().is_empty() {
        return Some(refusal(
            "term-label-empty",
            "label must be a non-empty noun phrase",
            Some(curie),
            vec![RemediationEntry {
                command: format!("dont define {curie} --label \"a <noun phrase>\" --doc \"...\""),
                description: "Supply a non-empty singular indefinite noun phrase".to_string(),
            }],
        ));
    }
    if shape.check_indefinite() && !label_has_indefinite_article(label) {
        let article = best_article_for(label);
        return Some(refusal(
            "term-shape-indefinite",
            "label must begin with 'a' or 'an' followed by a noun phrase (SK11 §2.1.1(i))",
            Some(curie),
            vec![RemediationEntry {
                command: format!("dont define {curie} --label \"{article} {label}\" --doc \"...\""),
                description: format!(
                    "Prepend '{article}' to form a singular indefinite noun phrase"
                ),
            }],
        ));
    }
    if shape.check_punctuated() && label_ends_with_sentence_punctuation(label) {
        let clean: String = label
            .chars()
            .rev()
            .skip_while(|c| matches!(c, '.' | '?' | '!' | ';' | ':') || c.is_ascii_whitespace())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        return Some(refusal(
            "term-shape-punctuated",
            "label must not end with sentence-closing punctuation (SK11 §2.1.1(iv))",
            Some(curie),
            vec![RemediationEntry {
                command: format!("dont define {curie} --label \"{clean}\" --doc \"...\""),
                description: "Remove the trailing punctuation from the label".to_string(),
            }],
        ));
    }
    if shape.check_compound() && label_compound_undeclared_with_markers(label, shape) {
        return Some(refusal(
            "term-compound-undeclared",
            "compound label requires a parenthesised variable list (SK11 §2.1.1(v))",
            Some(curie),
            vec![RemediationEntry {
                command: format!(
                    "dont define {curie} --label \"a pair (x, y) where x and y are integers\" --doc \"...\""
                ),
                description: "Add a parenthesised variable list after the compound marker"
                    .to_string(),
            }],
        ));
    }
    if shape.check_sentence() && label_contains_sentence_verb(label) {
        return Some(refusal(
            "term-label-sentence",
            "label contains a verb token outside a variable list or where-clause — rephrase as a noun phrase (SK11 §2.1)",
            Some(curie),
            vec![RemediationEntry {
                command: format!("dont define {curie} --label \"a <noun phrase>\" --doc \"...\""),
                description:
                    "Remove verb tokens (is, are, has, have, does, do, was, were) from the label"
                        .to_string(),
            }],
        ));
    }
    None
}

fn extract_doc_leading_phrase(doc: &str) -> String {
    let end = doc.find(['.', '?', '!', ';']).unwrap_or(doc.len());
    let phrase = doc[..end].trim();
    let token_capped: String = phrase
        .split_whitespace()
        .take(15)
        .collect::<Vec<_>>()
        .join(" ");
    token_capped.chars().take(80).collect()
}

fn doc_shape_warnings(doc: &str) -> Vec<Warning> {
    let phrase = extract_doc_leading_phrase(doc);
    if phrase.trim().is_empty() {
        return vec![];
    }
    if !label_has_indefinite_article(&phrase) {
        return vec![Warning {
            rule_name: "term-doc-shape-indefinite".to_string(),
            entity_id: None,
            message: "leading phrase of --doc does not begin with 'a' or 'an'; supply --label for a precise type-text".to_string(),
            suggested_remediation: Some(
                "Prepend 'a' or 'an' to the noun phrase or supply --label".to_string(),
            ),
        }];
    }
    if label_ends_with_sentence_punctuation(&phrase) {
        return vec![Warning {
            rule_name: "term-doc-shape-punctuated".to_string(),
            entity_id: None,
            message: "leading phrase of --doc ends with sentence punctuation; supply --label for a clean type-text".to_string(),
            suggested_remediation: Some(
                "Remove trailing punctuation or supply --label".to_string(),
            ),
        }];
    }
    if label_contains_sentence_verb(&phrase) {
        return vec![Warning {
            rule_name: "term-doc-shape-sentence".to_string(),
            entity_id: None,
            message: "leading phrase of --doc contains a verb token; supply --label for a precise noun phrase type-text".to_string(),
            suggested_remediation: Some(
                "Remove verb tokens from the phrase or supply --label".to_string(),
            ),
        }];
    }
    vec![]
}

fn nonfunctional_label_warning(label: &str, cfg: &TermNonfunctionalConfig) -> Option<Warning> {
    if cfg.matches_label(label) {
        Some(Warning {
            rule_name: "term-nonfunctional-label".to_string(),
            entity_id: None,
            message: "label suggests a non-functional relationship folded into the type; consider aspect-based decomposition".to_string(),
            suggested_remediation: Some(
                "Refactor the type so the relationship is expressed as an aspect rather than embedded in the label".to_string(),
            ),
        })
    } else {
        None
    }
}

fn main() {
    let cli = Cli::parse();

    // Resolve author: explicit flag > $DONT_AUTHOR > $USER
    let author = cli
        .author
        .clone()
        .or_else(|| std::env::var("DONT_AUTHOR").ok())
        .or_else(|| std::env::var("USER").ok());
    if let Some(a) = author {
        set_author(a);
    }

    if !cli.json {
        HUMAN_MODE.with(|m| m.set(true));
    }
    if (cli.plain || cli.no_color) && !cli.json {
        PLAIN_MODE.with(|m| m.set(true));
    }
    if cli.color && !cli.json {
        FORCE_COLOR_MODE.with(|m| m.set(true));
    }
    if cli.quiet && !cli.json {
        QUIET_MODE.with(|m| m.set(true));
    }
    if cli.no_persist || std::env::var("DONT_NO_PERSIST").as_deref() == Ok("1") {
        NO_PERSIST_MODE.with(|m| m.set(true));
    }

    // --version [--json]
    if cli.version {
        if cli.json {
            let env = Envelope::success(
                EnvelopeKind::Version,
                json!({
                    "version": CLI_VERSION,
                    "name": "dont",
                }),
                vec![],
                vec![],
            );
            println!("{}", serde_json::to_string(&env).unwrap());
        } else {
            println!("dont {CLI_VERSION}");
        }
        process::exit(0);
    }

    let command = match cli.command {
        Some(c) => c,
        None => {
            let _ = Cli::command().print_help();
            process::exit(0);
        }
    };

    // Detect which interface is active (dont vs dt) and enforce vocabulary separation.
    let iface = active_interface();
    let is_dt = iface == "dt";

    // Normalize canonical-name aliases to implementation equivalents and enforce
    // per-interface vocabulary. `dismiss` is the spec-canonical fourth core verb
    // (deprecated in v0.3 — use `flag`). The dt/dont vocabulary split is enforced here:
    // dont-exclusive: conclude, trust, forget
    // dt-exclusive:   record, challenge, lock
    let command = match command {
        Command::Dismiss {
            id,
            evidence,
            file,
            url,
            lines,
            anchor,
            excerpt,
        } => {
            eprintln!(
                "warning: `dismiss` is deprecated and will be removed in a future version. \
                 Use `flag` instead."
            );
            Command::Flag {
                id,
                evidence,
                file,
                url,
                lines,
                anchor,
                excerpt,
            }
        }
        // dt-exclusive: record, challenge, lock
        Command::Record {
            statement,
            depends_on,
            confidence,
        } => {
            if !is_dt {
                emit_error_and_exit(cross_vocab_refusal("record", "dont", "conclude"), vec![], 2);
            }
            Command::Conclude {
                statement,
                depends_on,
                confidence,
            }
        }
        Command::Challenge { id, reason } => {
            if !is_dt {
                emit_error_and_exit(cross_vocab_refusal("challenge", "dont", "trust"), vec![], 2);
            }
            Command::Trust { id, reason }
        }
        Command::Lock { id } => {
            if !is_dt {
                emit_error_and_exit(cross_vocab_refusal("lock", "dont", "forget"), vec![], 2);
            }
            Command::Forget { id }
        }
        // dont-exclusive: conclude, trust, forget (errors when invoked via dt)
        Command::Conclude {
            statement,
            depends_on,
            confidence,
        } => {
            if is_dt {
                emit_error_and_exit(cross_vocab_refusal("conclude", "dt", "record"), vec![], 2);
            }
            Command::Conclude {
                statement,
                depends_on,
                confidence,
            }
        }
        Command::Trust { id, reason } => {
            if is_dt {
                emit_error_and_exit(cross_vocab_refusal("trust", "dt", "challenge"), vec![], 2);
            }
            Command::Trust { id, reason }
        }
        Command::Forget { id } => {
            if is_dt {
                emit_error_and_exit(cross_vocab_refusal("forget", "dt", "lock"), vec![], 2);
            }
            Command::Forget { id }
        }
        other => other,
    };

    match command {
        Command::Init { strict } => {
            let mode = if strict {
                ProjectMode::Strict
            } else {
                ProjectMode::Permissive
            };
            match Project::init(&cwd(), mode) {
                Ok(_) => {
                    let env = Envelope::success(
                        EnvelopeKind::Empty,
                        json!({ "mode": mode.as_str() }),
                        vec![],
                        vec![HintEntry {
                            command: "dont conclude \"claim text\"".to_string(),
                            description: "Introduce your first claim".to_string(),
                        }],
                    );
                    emit_confirm_json(&env);
                }
                Err(err) => {
                    let (code, message, exit) = project_error_to_exit(&err);
                    let remediation = remediation_for_project_error(&err);
                    let err_result = ErrorResult {
                        code,
                        message,
                        rule_name: None,
                        spec_ref: None,
                        entity_id: None,
                        unmet_clauses: vec![],
                        remediation,
                    };
                    emit_error_and_exit(err_result, vec![], exit);
                }
            }
        }

        Command::Conclude {
            statement,
            depends_on,
            confidence,
        } => {
            let project = open_project_or_exit();

            if statement == "-" {
                emit_error_and_exit(
                    refusal(
                        "stdin-not-supported",
                        "conclude does not read entity IDs from stdin; provide the statement as an argument",
                        None,
                        vec![RemediationEntry {
                            command: "dont conclude \"<claim text>\"".to_string(),
                            description: "Provide the statement directly as an argument"
                                .to_string(),
                        }],
                    ),
                    vec![],
                    2,
                );
            }

            if statement.trim().is_empty() {
                emit_error_and_exit(
                    refusal(
                        "empty-statement",
                        "statement: required; expected non-empty claim text",
                        None,
                        vec![RemediationEntry {
                            command: "dont conclude \"<claim text>\"".to_string(),
                            description: "Provide a non-empty statement that can be grounded"
                                .to_string(),
                        }],
                    ),
                    vec![],
                    1,
                );
            }

            if let Some(err) = validate_claim_statement(&statement, "dont conclude") {
                emit_error_and_exit(err, vec![], 1);
            }

            let mut resolved_depends_on: Vec<String> = vec![];
            let mut unresolved: Vec<String> = vec![];
            for dep in &depends_on {
                if dep.starts_with("term:") {
                    match project.store.term_by_id(dep) {
                        Ok(Some(_)) => resolved_depends_on.push(dep.clone()),
                        Ok(None) => unresolved.push(dep.clone()),
                        Err(err) => handle_store_error(err, None),
                    }
                } else {
                    match project.store.resolve_curie_reference(dep) {
                        Ok(Some(CurieResolution::Coined(term))) => {
                            resolved_depends_on.push(term.id)
                        }
                        Ok(Some(CurieResolution::Imported(_))) => {
                            resolved_depends_on.push(dep.clone())
                        }
                        Ok(None) => unresolved.push(dep.clone()),
                        Err(err) => handle_store_error(err, None),
                    }
                }
            }

            let is_strict = project.mode() == Some(ProjectMode::Strict);
            if is_strict && !unresolved.is_empty() {
                let list = unresolved.join(", ");
                emit_error_and_exit(
                    refusal(
                        "unresolved-term-ref",
                        &format!("strict mode: unresolved term references: {list}"),
                        None,
                        unresolved
                            .iter()
                            .map(|c| RemediationEntry {
                                command: if c.starts_with("term:") {
                                    "dont vocab".to_string()
                                } else {
                                    format!("dont define {c} --doc \"<definition>\"")
                                },
                                description: if c.starts_with("term:") {
                                    format!("List terms and confirm whether {c} exists")
                                } else {
                                    format!("Define the term {c} before concluding")
                                },
                            })
                            .collect(),
                    ),
                    vec![],
                    1,
                );
            }

            let warnings: Vec<Warning> = unresolved
                .iter()
                .map(|c| Warning {
                    rule_name: "unresolved-term-ref".to_string(),
                    entity_id: None,
                    message: format!(
                        "term reference {c} is not yet defined; verification blocked until resolved"
                    ),
                    suggested_remediation: Some(if c.starts_with("term:") {
                        "dont vocab".to_string()
                    } else {
                        format!("dont define {c} --doc \"<definition>\"")
                    }),
                })
                .collect();

            let all_depends_on: Vec<String> = resolved_depends_on
                .iter()
                .chain(unresolved.iter())
                .cloned()
                .collect();

            if no_persist_mode() {
                // Validate dedup without writing.
                if let Err(err) = project.store.check_claim_dedup(&statement) {
                    handle_store_error(err, None);
                }
                let now = dont::store::now_rfc3339_pub();
                let payload = json!({
                    "id": "ephemeral",
                    "entity_kind": "claim",
                    "statement": statement,
                    "status": "unverified",
                    "derived_assessments": [],
                    "atoms": [],
                    "hypotheses": [],
                    "evidence": [],
                    "depends_on": all_depends_on,
                    "applicable_rules": {},
                    "created_at": now,
                });
                let env = Envelope::success(EnvelopeKind::Claim, payload, warnings, vec![]);
                emit_confirm_json(&env);
            } else {
                match project
                    .store
                    .append_claim(&statement, &all_depends_on, confidence)
                {
                    Ok(result) => {
                        let payload = json!({
                            "id": result.id,
                            "entity_kind": "claim",
                            "statement": statement,
                            "status": "unverified",
                            "derived_assessments": [],
                            "atoms": [],
                            "hypotheses": [],
                            "evidence": [],
                            "depends_on": all_depends_on,
                            "applicable_rules": {},
                            "created_at": result.created_at,
                        });
                        let env = Envelope::success_with_tx(
                            EnvelopeKind::Claim,
                            payload,
                            warnings,
                            vec![HintEntry {
                                command: format!("dont show {}", result.id),
                                description: "Inspect the new claim".to_string(),
                            }],
                            Some(result.tx as u64),
                        );
                        emit_confirm_json(&env);
                    }
                    Err(err) => handle_store_error(err, None),
                }
            }
        }

        Command::Define { curie, doc, label } => {
            let curie = match curie {
                Some(curie) => curie,
                None => emit_error_and_exit(
                    refusal(
                        "curie-required",
                        "define requires a CURIE such as proj:TermName",
                        None,
                        vec![RemediationEntry {
                            command: "dont define proj:TermName --doc \"<definition>\"".to_string(),
                            description: "Re-run with the term CURIE as the first argument"
                                .to_string(),
                        }],
                    ),
                    vec![],
                    1,
                ),
            };
            let doc = match doc {
                Some(doc) if !doc.trim().is_empty() => doc,
                _ => emit_error_and_exit(
                    refusal(
                        "doc-required",
                        "--doc: required; expected non-empty prose definition",
                        None,
                        vec![RemediationEntry {
                            command: format!("dont define {curie} --doc \"<definition>\""),
                            description: "Re-run with a concise prose definition".to_string(),
                        }],
                    ),
                    vec![],
                    1,
                ),
            };

            let project = open_project_or_exit();
            let config = project.load_config();

            let warnings = match &label {
                Some(lbl) => {
                    if let Some(err) = validate_label(lbl, &curie, &config.define.shape) {
                        emit_error_and_exit(err, vec![], 1);
                    }
                    nonfunctional_label_warning(lbl, &config.rules.term_nonfunctional)
                        .into_iter()
                        .collect()
                }
                None => doc_shape_warnings(&doc),
            };
            if no_persist_mode() {
                let fake_result = dont::store::AppendResult {
                    id: format!("term:{curie}"),
                    event_id: String::new(),
                    tx: 0,
                    created_at: dont::store::now_rfc3339_pub(),
                };
                let fake_term = dont::store::TermRecord {
                    id: fake_result.id.clone(),
                    curie: curie.clone(),
                    label: label.clone(),
                    definition: doc.clone(),
                    status: dont::model::Status::Unverified,
                    created_at: fake_result.created_at.clone(),
                    events: vec![],
                };
                emit_term_view(&fake_term, &fake_result, &project.store, warnings);
            } else {
                let result = match project.store.append_term(&curie, &doc, label.as_deref()) {
                    Ok(result) => result,
                    Err(err) => handle_store_error(err, None),
                };
                let term = match project.store.term_by_id(&result.id) {
                    Ok(Some(term)) => term,
                    Ok(None) => handle_store_error(
                        StoreError::Malformed(format!("term {} vanished after define", result.id)),
                        Some(&result.id),
                    ),
                    Err(err) => handle_store_error(err, Some(&result.id)),
                };
                emit_term_view(&term, &result, &project.store, warnings);
            }
        }

        Command::Trust { id, reason } => {
            let reason = match reason {
                None => {
                    emit_error_and_exit(
                        refusal(
                            "reason-required",
                            "--reason: required; expected specific grounds for doubt (not a hedge)",
                            Some(&id),
                            vec![RemediationEntry {
                                command: format!("dont trust {id} --reason \"<specific grounds>\""),
                                description: "Re-run with a concrete, non-hedged reason"
                                    .to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    );
                }
                Some(r) => r,
            };

            if let Some(err) = validate_claim_statement(&reason, "dont trust") {
                emit_error_and_exit(err, vec![], 1);
            }

            let project = open_project_or_exit();
            let config = project.load_config();

            if contains_hedge(&reason, &config.trust.hedges.patterns) {
                emit_error_and_exit(
                    refusal(
                        "reason-not-hedge",
                        "reason contains an epistemic hedge — state the specific grounds for doubt",
                        Some(&id),
                        vec![RemediationEntry {
                            command: format!("dont trust {id} --reason \"<specific grounds>\""),
                            description: "Replace the hedge with a concrete reason".to_string(),
                        }],
                    ),
                    vec![],
                    1,
                );
            }
            let event = StoreEvent {
                kind: StoreEventKind::Trusted,
                note: Some(reason),
                evidence: vec![],
            };
            if let EntityId::Term(_) = EntityId::parse(&id) {
                apply_term_transition(&project, &id, model_trust, event, "term-not-found", "trust");
                return;
            }
            apply_claim_transition(
                &project,
                &id,
                model_trust,
                event,
                "claim-not-found",
                "trust",
            );
        }

        Command::Forget { id } => {
            let project = open_project_or_exit();
            run_per_entity(id, |id| {
                if let EntityId::Term(_) = EntityId::parse(id) {
                    return emit_error_no_exit(
                        refusal(
                            "wrong-entity-kind",
                            "forget (lock) applies to claims only; terms cannot be locked",
                            Some(id),
                            vec![RemediationEntry {
                                command: format!("dont show {id}"),
                                description: "Inspect the term instead of trying to lock it"
                                    .to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    );
                }

                let record = match project.store.claim_by_id(id) {
                    Ok(Some(r)) => r,
                    Ok(None) => {
                        return emit_error_no_exit(
                            refusal(
                                "claim-not-found",
                                &format!("no claim with id {id}"),
                                Some(id),
                                vec![RemediationEntry {
                                    command: "dont list".to_string(),
                                    description: "List all claims to find the correct id"
                                        .to_string(),
                                }],
                            ),
                            vec![],
                            1,
                        );
                    }
                    Err(err) => return handle_store_error_code(err, Some(id)),
                };

                let current = record.status;
                match current {
                    Status::Locked => {
                        return emit_error_no_exit(
                            refusal(
                                "claim-locked",
                                "claim is already locked",
                                Some(id),
                                vec![RemediationEntry {
                                    command: format!("dont show {id}"),
                                    description: "Inspect the locked claim".to_string(),
                                }],
                            ),
                            vec![],
                            1,
                        );
                    }
                    Status::Verified => {}
                    _ => {
                        return emit_error_no_exit(
                            refusal(
                                "claim-not-verified",
                                "claim must be verified before it can be locked",
                                Some(id),
                                vec![RemediationEntry {
                                    command: format!("dont dismiss {id} --evidence <uri>"),
                                    description: "Attach evidence until the claim reaches verified"
                                        .to_string(),
                                }],
                            ),
                            vec![],
                            1,
                        );
                    }
                }

                let unmet_clauses = lockable_unmet_clauses(&record, &project.store);
                if !unmet_clauses.is_empty() {
                    let err_result = ErrorResult::new(
                        "rule-not-met",
                        "lockable gate is not met",
                        Some("lockable"),
                        None,
                        Some(id),
                        unmet_clauses,
                        vec![RemediationEntry {
                            command: format!("dont show {id}"),
                            description: "Inspect the claim and satisfy the unmet lock gates"
                                .to_string(),
                        }],
                    )
                    .expect("lock refusal must include remediation");
                    return emit_error_no_exit(err_result, vec![], 1);
                }

                apply_claim_transition(
                    &project,
                    id,
                    model_lock,
                    StoreEvent {
                        kind: StoreEventKind::Locked,
                        note: None,
                        evidence: vec![],
                    },
                    "claim-not-found",
                    "lock",
                );
                0
            });
        }

        Command::Reopen { id } => {
            let project = open_project_or_exit();

            let event = StoreEvent {
                kind: StoreEventKind::Reopened,
                note: None,
                evidence: vec![],
            };
            if let EntityId::Term(_) = EntityId::parse(&id) {
                apply_term_transition(
                    &project,
                    &id,
                    model_reopen,
                    event,
                    "entity-not-found",
                    "reopen",
                );
            } else {
                apply_claim_transition(
                    &project,
                    &id,
                    model_reopen,
                    event,
                    "entity-not-found",
                    "reopen",
                );
            }
        }

        Command::Ignore { id, reason } => {
            let reason = match reason {
                None => emit_error_and_exit(
                    refusal(
                        "reason-required",
                        "--reason: required; expected explanation for why this entity is being set aside",
                        Some(&id),
                        vec![RemediationEntry {
                            command: format!("dont ignore {id} --reason \"<substantive reason>\""),
                            description: "Re-run with a concrete, non-hedged reason".to_string(),
                        }],
                    ),
                    vec![],
                    1,
                ),
                Some(r) => r,
            };

            let project = open_project_or_exit();
            let config = project.load_config();

            if contains_hedge(&reason, &config.trust.hedges.patterns) {
                emit_error_and_exit(
                    refusal(
                        "reason-not-hedge",
                        "reason contains an epistemic hedge — state the specific grounds for ignoring",
                        Some(&id),
                        vec![RemediationEntry {
                            command: format!("dont ignore {id} --reason \"<substantive reason>\""),
                            description: "Replace the hedge with a concrete reason".to_string(),
                        }],
                    ),
                    vec![],
                    1,
                );
            }

            let event = StoreEvent {
                kind: StoreEventKind::Ignored,
                note: Some(reason),
                evidence: vec![],
            };
            if let EntityId::Term(_) = EntityId::parse(&id) {
                apply_term_transition(
                    &project,
                    &id,
                    model_ignore,
                    event,
                    "entity-not-found",
                    "ignore",
                );
            } else {
                apply_claim_transition(
                    &project,
                    &id,
                    model_ignore,
                    event,
                    "entity-not-found",
                    "ignore",
                );
            }
        }

        Command::Flag {
            id,
            evidence,
            file,
            url,
            lines,
            anchor,
            excerpt,
        } => {
            if evidence.is_empty() && file.is_none() && url.is_none() {
                emit_error_and_exit(
                    refusal(
                        "no-evidence",
                        "--evidence: required; expected at least one URI, --file locator, or --url permalink",
                        Some(&id),
                        vec![RemediationEntry {
                            command: format!("dont flag {id} --evidence <uri>"),
                            description: "Re-run with at least one evidence reference".to_string(),
                        }],
                    ),
                    vec![],
                    1,
                );
            }

            // Validate all URI strings before opening the project so that a
            // malformed locator fails fast without side effects.
            for uri in &evidence {
                if let Err(msg) = validate_evidence_uri(uri) {
                    emit_error_and_exit(
                        refusal(
                            "malformed-evidence-uri",
                            &msg,
                            Some(&id),
                            vec![RemediationEntry {
                                command: format!("dont flag {id} --evidence <http://...>"),
                                description:
                                    "Use a valid http:// or https:// URI as the evidence reference"
                                        .to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    );
                }
            }

            let project = open_project_or_exit();
            let project_root = project
                .dont_dir
                .parent()
                .unwrap_or(&project.dont_dir)
                .to_path_buf();

            // Build the full evidence list, appending structured locator if --file or --url was given.
            let mut all_evidence: Vec<Value> = evidence.into_iter().map(Value::String).collect();
            if let Some(ref file_path) = file {
                all_evidence.push(resolve_file_locator(
                    file_path,
                    lines.as_deref(),
                    anchor.as_deref(),
                    excerpt.as_deref(),
                    &project_root,
                    Some(&id),
                    &format!("dont flag {id}"),
                ));
            }
            if let Some(ref url_str) = url {
                all_evidence.push(resolve_url_locator(
                    url_str,
                    lines.as_deref(),
                    anchor.as_deref(),
                    excerpt.as_deref(),
                ));
            }
            // Terms don't have depends_on fields so no dependency gate is needed here.
            // If terms gain dependencies in the future, add dependency_gate_unmet_clauses.
            if let EntityId::Term(_) = EntityId::parse(&id) {
                apply_term_transition(
                    &project,
                    &id,
                    model_flag,
                    StoreEvent {
                        kind: StoreEventKind::Flagged,
                        note: None,
                        evidence: all_evidence.clone(),
                    },
                    "term-not-found",
                    "flag",
                );
                return;
            }

            let record = match project.store.claim_by_id(&id) {
                Ok(Some(r)) => r,
                Ok(None) => emit_error_and_exit(
                    transition_not_found_refusal("claim-not-found", &id),
                    vec![],
                    1,
                ),
                Err(err) => handle_store_error(err, Some(&id)),
            };

            let dependency_unmet = dependency_gate_unmet_clauses(&record, &project.store);
            if !dependency_unmet.is_empty() {
                let rule_name = dependency_gate_rule_name(&dependency_unmet);
                let err_result = ErrorResult::new(
                    "rule-not-met",
                    "dependency integrity blocks verification",
                    Some(rule_name),
                    None,
                    Some(&id),
                    dependency_unmet,
                    vec![RemediationEntry {
                        command: format!("dont show {id}"),
                        description: "Inspect the blocking dependency assessments".to_string(),
                    }],
                )
                .expect("dependency gate refusal must include remediation");
                emit_error_and_exit(err_result, vec![], 1);
            }

            apply_claim_transition_or_append_evidence(
                &project,
                &id,
                model_flag,
                StoreEvent {
                    kind: StoreEventKind::Flagged,
                    note: None,
                    evidence: all_evidence,
                },
                "claim-not-found",
                "flag",
            );
        }

        Command::Undoubt { id } => {
            let project = open_project_or_exit();

            let event = StoreEvent {
                kind: StoreEventKind::Undoubted,
                note: None,
                evidence: vec![],
            };
            if let EntityId::Term(_) = EntityId::parse(&id) {
                apply_term_transition(
                    &project,
                    &id,
                    model_undoubt,
                    event,
                    "entity-not-found",
                    "undoubt",
                );
            } else {
                apply_claim_transition(
                    &project,
                    &id,
                    model_undoubt,
                    event,
                    "entity-not-found",
                    "undoubt",
                );
            }
        }

        Command::Show { id, history } => {
            let project = open_project_or_exit();
            run_per_entity(id, |id| match project.store.resolve_entity(id) {
                Ok(Some(EntityResolution::Claim(record))) => {
                    let payload = build_claim_show_view(&record, &project.store, history);
                    let env = Envelope::success(
                        EnvelopeKind::Claim,
                        payload,
                        vec![],
                        vec![HintEntry {
                            command: format!("dont trust {} --reason \"...\"", record.id),
                            description: "Register doubt about this claim".to_string(),
                        }],
                    );
                    emit_json(&env);
                    0
                }
                Ok(Some(EntityResolution::Term(record))) => {
                    let payload = build_term_view(&record, &project.store);
                    let env = Envelope::success(
                        EnvelopeKind::Term,
                        payload,
                        vec![],
                        vec![HintEntry {
                            command: format!("dont trust {} --reason \"...\"", record.id),
                            description: "Register doubt about this term".to_string(),
                        }],
                    );
                    emit_json(&env);
                    0
                }
                Ok(None) => {
                    let (code, message, remediation) = entity_not_found_error(id);
                    emit_error_no_exit(refusal(code, &message, Some(id), remediation), vec![], 1)
                }
                Err(err) => handle_store_error_code(err, Some(id)),
            });
        }

        Command::Why { id } => {
            let project = open_project_or_exit();
            run_per_entity(id, |id| match project.store.resolve_entity(id) {
                Ok(Some(EntityResolution::Claim(record))) => {
                    let payload = build_claim_why_view(&record, &project.store);
                    let env = Envelope::success(
                        EnvelopeKind::Why,
                        payload,
                        vec![],
                        vec![HintEntry {
                            command: format!("dont show {}", record.id),
                            description: "Inspect the current claim view".to_string(),
                        }],
                    );
                    emit_json(&env);
                    0
                }
                Ok(Some(EntityResolution::Term(record))) => {
                    let payload = build_term_why_view(&record, &project.store);
                    let env = Envelope::success(
                        EnvelopeKind::Why,
                        payload,
                        vec![],
                        vec![HintEntry {
                            command: format!("dont show {}", record.id),
                            description: "Inspect the current term view".to_string(),
                        }],
                    );
                    emit_json(&env);
                    0
                }
                Ok(None) => {
                    let (code, message, remediation) = entity_not_found_error(id);
                    emit_error_no_exit(refusal(code, &message, Some(id), remediation), vec![], 1)
                }
                Err(err) => handle_store_error_code(err, Some(id)),
            });
        }

        Command::VerifyEvidence {
            id,
            timeout_seconds,
        } => {
            let project = open_project_or_exit();
            let config = project.load_config();
            let effective_timeout = timeout_seconds.or(config.verify_evidence.default_timeout_s);

            let (entity_kind, status, evidence) = if let EntityId::Term(_) = EntityId::parse(&id) {
                match project.store.term_by_id(&id) {
                    Ok(Some(record)) => (
                        EnvelopeKind::Term,
                        format!("{:?}", record.status).to_lowercase(),
                        collect_term_evidence(&record),
                    ),
                    Ok(None) => emit_error_and_exit(
                        refusal(
                            "entity-not-found",
                            &format!("no entity with id {id}"),
                            Some(&id),
                            vec![RemediationEntry {
                                command: "dont vocab".to_string(),
                                description: "List terms to find the correct id".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                    Err(err) => handle_store_error(err, Some(&id)),
                }
            } else {
                match project.store.claim_by_id(&id) {
                    Ok(Some(record)) => (
                        EnvelopeKind::Claim,
                        format!("{:?}", record.status).to_lowercase(),
                        collect_evidence(&record),
                    ),
                    Ok(None) => emit_error_and_exit(
                        refusal(
                            "entity-not-found",
                            &format!("no entity with id {id}"),
                            Some(&id),
                            vec![RemediationEntry {
                                command: "dont list".to_string(),
                                description: "List claims to find the correct id".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                    Err(err) => handle_store_error(err, Some(&id)),
                }
            };

            if evidence.is_empty() {
                let remediation = if entity_kind == EnvelopeKind::Claim {
                    RemediationEntry {
                        command: format!("dont dismiss {id} --evidence <uri>"),
                        description: "Attach evidence to the claim before verifying liveness"
                            .to_string(),
                    }
                } else {
                    RemediationEntry {
                        command: format!("dont show {id}"),
                        description: "Inspect the term and confirm evidence has been attached before verifying liveness".to_string(),
                    }
                };
                emit_error_and_exit(
                    refusal(
                        "no-evidence",
                        "verify-evidence requires at least one attached evidence reference",
                        Some(&id),
                        vec![remediation],
                    ),
                    vec![],
                    1,
                );
            }

            let mocks = mocked_evidence_outcomes();
            let project_root = project_root_from_store(&project.store);
            let uri_results: Vec<EvidenceCheckResult> = evidence
                .iter()
                .filter_map(|v| v.as_str())
                .map(|uri| check_evidence_uri(uri, mocks.as_ref(), effective_timeout))
                .collect();
            let warnings: Vec<Warning> = uri_results
                .iter()
                .filter_map(|result| evidence_check_warning(&id, result))
                .collect();
            let mut results: Vec<Value> = uri_results
                .iter()
                .map(|result| {
                    serde_json::to_value(result).expect("evidence check result serializes")
                })
                .collect();
            for locator in evidence.iter().filter(|v| {
                v.as_object()
                    .and_then(|obj| obj.get("kind"))
                    .and_then(Value::as_str)
                    == Some("repo-file")
            }) {
                let audit = locator_audit(locator, &project_root);
                let outcome = audit
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("unchecked");
                let mut result = json!({
                    "locator": project_evidence_entry(locator, &project_root),
                    "outcome": outcome,
                });
                if let Some(detail) = audit.get("detail").and_then(Value::as_str) {
                    result["detail"] = Value::String(detail.to_string());
                }
                results.push(result);
            }
            let payload = json!({
                "entity_id": id,
                "entity_kind": entity_kind,
                "status": status,
                "timeout_seconds": effective_timeout,
                "results": results,
            });
            let env = Envelope::success(
                EnvelopeKind::EvidenceCheck,
                payload,
                warnings,
                vec![HintEntry {
                    command: format!("dont show {id}"),
                    description: "Inspect the unchanged entity status and attached evidence"
                        .to_string(),
                }],
            );
            emit_json(&env);
        }

        Command::Prime => {
            let project = open_project_or_exit();
            let prime_config = project.load_config();
            let claims = match project.store.list_claims() {
                Ok(c) => c,
                Err(err) => handle_store_error(err, None),
            };
            let terms = match project.store.list_terms() {
                Ok(t) => t,
                Err(err) => handle_store_error(err, None),
            };
            let mut unverified = 0;
            let mut doubted = 0;
            let mut verified = 0;
            let mut ignored = 0;
            let mut locked = 0;
            let mut blocking = Vec::new();
            let mut ac_stale = 0u32;
            let mut ac_compromised = 0u32;
            let mut ac_dangling = 0u32;
            let mut ac_unresolved = 0u32;
            let mut ac_drifted_evidence = 0u32;
            let project_root = project_root_from_store(&project.store);
            for claim in &claims {
                match claim.status {
                    Status::Unverified => unverified += 1,
                    Status::Doubted => {
                        doubted += 1;
                        blocking.push(json!({
                            "id": claim.id,
                            "statement": claim.statement,
                            "status": "doubted",
                        }));
                    }
                    Status::Verified => verified += 1,
                    Status::Ignored => ignored += 1,
                    Status::Locked => locked += 1,
                }
                for a in derived_assessments_for_claim(claim, &project.store) {
                    match a.as_str() {
                        "stale" => ac_stale += 1,
                        "compromised-support" => ac_compromised += 1,
                        "dangling-dependency" => ac_dangling += 1,
                        "unresolved-term" => ac_unresolved += 1,
                        _ => {}
                    }
                }
                let projected = project_evidence(collect_evidence(claim), &project_root);
                if projected.iter().any(|e| {
                    e.get("audit")
                        .and_then(|a| a.get("status"))
                        .and_then(|s| s.as_str())
                        == Some("drifted")
                }) {
                    ac_drifted_evidence += 1;
                }
            }
            for term in &terms {
                match term.status {
                    Status::Unverified => unverified += 1,
                    Status::Doubted => {
                        doubted += 1;
                        blocking.push(json!({
                            "id": term.id,
                            "curie": term.curie,
                            "status": "doubted",
                        }));
                    }
                    Status::Verified => verified += 1,
                    Status::Ignored => ignored += 1,
                    Status::Locked => locked += 1,
                }
                let projected = project_evidence(collect_term_evidence(term), &project_root);
                if projected.iter().any(|e| {
                    e.get("audit")
                        .and_then(|a| a.get("status"))
                        .and_then(|s| s.as_str())
                        == Some("drifted")
                }) {
                    ac_drifted_evidence += 1;
                }
            }
            let payload = json!({
                "project": "dont-project",
                "mode": project.mode().map(ProjectMode::as_str),
                "status_counts": {
                    "unverified": unverified,
                    "doubted": doubted,
                    "verified": verified,
                    "locked": locked,
                    "ignored": ignored,
                },
                "assessment_counts": {
                    "stale": ac_stale,
                    "compromised-support": ac_compromised,
                    "dangling-dependency": ac_dangling,
                    "unresolved-term": ac_unresolved,
                    "drifted_evidence": ac_drifted_evidence,
                },
                "rules": { "strict": prime_config.rules.strict, "warn": prime_config.rules.warn },
                "ontologies": [],
                "blocking": blocking,
                "pending_spawns": 0,
                "harness_mode": false,
                "invariants": [
                    "Use --json envelopes for agent-facing commands",
                    "Verified entities must not depend on unresolved terms"
                ],
            });
            let env = Envelope::success(EnvelopeKind::Prime, payload, vec![], vec![]);
            emit_json(&env);
            if !blocking.is_empty() {
                std::process::exit(1);
            }
        }

        Command::Doctor { strict, fix } => {
            let project = open_project_or_exit();
            if fix && let Err(err) = project.refresh_managed_docs() {
                emit_project_error_and_exit(&err);
            }
            if fix && let Err(err) = project.refresh_managed_skill_packs() {
                emit_project_error_and_exit(&err);
            }

            let (managed_clean, managed_details) = match project.managed_docs_status() {
                Ok(status) => status,
                Err(err) => emit_project_error_and_exit(&err),
            };
            let pack_health = match project.managed_skill_packs_status() {
                Ok(h) => h,
                Err(err) => emit_project_error_and_exit(&err),
            };

            let managed_status = if managed_clean { "pass" } else { "warn" };
            let managed_detail = if managed_clean {
                "managed docs are current".to_string()
            } else {
                managed_details.join("; ")
            };
            let skills_all_pass = pack_health
                .iter()
                .all(|h| h.state == skill_pack::PackState::Pass);
            let skills_status = if skills_all_pass {
                "pass"
            } else if pack_health
                .iter()
                .any(|h| h.state == skill_pack::PackState::Missing)
            {
                "missing"
            } else {
                "stale"
            };
            let skills_detail = if skills_all_pass || pack_health.is_empty() {
                "managed skill packs are current".to_string()
            } else {
                pack_health
                    .iter()
                    .filter(|h| h.state != skill_pack::PackState::Pass)
                    .map(|h| h.detail.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            };
            let seed_snapshot_exists = project.seed_snapshot_path().is_file();
            let seed_snapshot_status = if seed_snapshot_exists { "pass" } else { "warn" };
            let seed_snapshot_detail = if seed_snapshot_exists {
                "seed snapshot is present".to_string()
            } else {
                format!(
                    "seed snapshot {} is missing; run dont init to repair the project layout",
                    project.seed_snapshot_path().display()
                )
            };
            let linkml_available = linkml_is_on_path();
            let linkml_available_status = if linkml_available { "pass" } else { "warn" };
            let linkml_available_detail = if linkml_available {
                "linkml is available on PATH".to_string()
            } else {
                "linkml is not on PATH; import linkml uses in-process parsing only — install linkml for secondary validation".to_string()
            };
            let checks = vec![
                json!({"name": "substrate", "status": "pass", "detail": "store opened successfully"}),
                json!({"name": "rules_compile", "status": "pass", "detail": "built-in rules available"}),
                json!({"name": "seed_snapshot", "status": seed_snapshot_status, "detail": seed_snapshot_detail}),
                json!({"name": "pending_spawns", "status": "pass", "detail": if project.root_doc_paths().is_empty() { "no pending spawn audit implemented; direct DONT_DIR override skips separate root managed docs" } else { "no pending spawn audit implemented" }}),
                json!({"name": "remediation_invariant", "status": "pass", "detail": "error remediation invariant available"}),
                json!({"name": "managed_docs", "status": managed_status, "detail": managed_detail}),
                json!({"name": "managed_skills", "status": skills_status, "detail": skills_detail}),
                json!({"name": "linkml_available", "status": linkml_available_status, "detail": linkml_available_detail}),
            ];
            let pass = checks.iter().filter(|c| c["status"] == "pass").count();
            let warn = checks.iter().filter(|c| c["status"] == "warn").count();
            let fail = checks.iter().filter(|c| c["status"] == "fail").count();
            let payload = json!({
                "cli_version": CLI_VERSION,
                "checks": checks,
                "summary": {"pass": pass, "warn": warn, "fail": fail},
            });
            let env = Envelope::success(EnvelopeKind::Doctor, payload, vec![], vec![]);
            emit_json(&env);
            // Emit ungrounded rejection events when DONT_EMIT_EVENTS=1.
            emit_ungrounded_events_if_enabled(&project);
            let exit_code = if strict {
                if warn > 0 || fail > 0 { 1 } else { 0 }
            } else if fail > 0 {
                1
            } else {
                0
            };
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
        }

        Command::List {
            status,
            kind,
            all,
            derived_assessment,
            as_of,
        } => {
            // Validate --as-of if supplied; emit structured error before opening the store.
            if let Some(ref raw) = as_of {
                if !is_valid_as_of(raw) {
                    emit_error_and_exit(
                        refusal(
                            "invalid-timestamp",
                            &format!(
                                "invalid --as-of value '{raw}'; expected RFC 3339 datetime or YYYY-MM-DD date"
                            ),
                            None,
                            vec![RemediationEntry {
                                command: "dont list --as-of 2026-01-01".to_string(),
                                description:
                                    "Use an ISO 8601 date or RFC 3339 datetime, e.g. 2026-01-01"
                                        .to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    );
                }
                // Historical snapshot queries are not yet implemented.
                emit_error_and_exit(
                    refusal(
                        "not-yet-implemented",
                        "historical snapshot queries (--as-of) are not yet implemented",
                        None,
                        vec![RemediationEntry {
                            command: "dont list --json".to_string(),
                            description: "Omit --as-of to query the current state".to_string(),
                        }],
                    ),
                    vec![],
                    1,
                );
            }
            let project = open_project_or_exit();
            let status_filter = match status {
                Some(raw) => match parse_claim_status_filter(&raw) {
                    Some(status) => Some(status),
                    None => emit_error_and_exit(
                        refusal(
                            "invalid-status",
                            &format!(
                                "unsupported claim status '{raw}'; expected one of: unverified, verified, doubted, ignored, locked"
                            ),
                            None,
                            vec![RemediationEntry {
                                command: "dont list --status unverified".to_string(),
                                description:
                                    "Use one of: unverified, verified, doubted, ignored, locked"
                                        .to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                },
                None => None,
            };
            let default_kind = kind.is_none() && !all;
            let list_kind = if all {
                ListKind::All
            } else {
                match kind {
                    Some(raw) => match parse_list_kind(&raw) {
                        Some(kind) => kind,
                        None => emit_error_and_exit(
                            refusal(
                                "invalid-kind",
                                &format!(
                                    "unsupported list kind '{raw}'; expected one of: claims, terms"
                                ),
                                None,
                                vec![RemediationEntry {
                                    command: "dont list --kind terms".to_string(),
                                    description: "Use one of: claims, terms".to_string(),
                                }],
                            ),
                            vec![],
                            1,
                        ),
                    },
                    None => ListKind::Claims,
                }
            };
            match list_kind {
                ListKind::Claims => {
                    let mut claims = match project.store.list_claims() {
                        Ok(c) => c,
                        Err(err) => handle_store_error(err, None),
                    };
                    if let Some(status_filter) = status_filter {
                        claims.retain(|claim| claim.status == status_filter);
                    }
                    if let Some(ref assessment) = derived_assessment {
                        claims.retain(|claim| {
                            derived_assessments_for_claim(claim, &project.store)
                                .iter()
                                .any(|a| a == assessment)
                        });
                    }
                    // Sort by created_at descending; use id (ULID) as tiebreaker within same second
                    claims.sort_by(|a, b| {
                        b.created_at
                            .cmp(&a.created_at)
                            .then_with(|| b.id.cmp(&a.id))
                    });
                    let views: Vec<Value> = claims
                        .iter()
                        .map(|c| build_claim_view(c, &project.store))
                        .collect();
                    let hints = match project.store.list_terms() {
                        Ok(terms) if default_kind && !terms.is_empty() => vec![HintEntry {
                            command: "dont list --kind terms".to_string(),
                            description: "List defined term entities as well".to_string(),
                        }],
                        Ok(_) => vec![],
                        Err(err) => handle_store_error(err, None),
                    };
                    let count = views.len();
                    let payload = json!({
                        "as_of": chrono::Utc::now().to_rfc3339(),
                        "count": count,
                        "claims": views,
                    });
                    let env = Envelope::success(EnvelopeKind::Claims, payload, vec![], hints);
                    emit_json(&env);
                }
                ListKind::Terms => {
                    let mut terms = match project.store.list_terms() {
                        Ok(t) => t,
                        Err(err) => handle_store_error(err, None),
                    };
                    if let Some(status_filter) = status_filter {
                        terms.retain(|term| term.status == status_filter);
                    }
                    terms.sort_by(|a, b| {
                        b.created_at
                            .cmp(&a.created_at)
                            .then_with(|| b.id.cmp(&a.id))
                    });
                    let views: Vec<Value> = terms
                        .iter()
                        .map(|term| build_term_view(term, &project.store))
                        .collect();
                    let env = Envelope::success(EnvelopeKind::TermList, views, vec![], vec![]);
                    emit_json(&env);
                }
                ListKind::All => {
                    let mut claims = match project.store.list_claims() {
                        Ok(c) => c,
                        Err(err) => handle_store_error(err, None),
                    };
                    if let Some(status_filter) = status_filter {
                        claims.retain(|claim| claim.status == status_filter);
                    }
                    claims.sort_by(|a, b| {
                        b.created_at
                            .cmp(&a.created_at)
                            .then_with(|| b.id.cmp(&a.id))
                    });
                    let claim_views: Vec<Value> = claims
                        .iter()
                        .map(|c| build_claim_view(c, &project.store))
                        .collect();
                    let mut terms = match project.store.list_terms() {
                        Ok(t) => t,
                        Err(err) => handle_store_error(err, None),
                    };
                    if let Some(status_filter) = status_filter {
                        terms.retain(|term| term.status == status_filter);
                    }
                    terms.sort_by(|a, b| {
                        b.created_at
                            .cmp(&a.created_at)
                            .then_with(|| b.id.cmp(&a.id))
                    });
                    let term_views: Vec<Value> = terms
                        .iter()
                        .map(|term| build_term_view(term, &project.store))
                        .collect();
                    let payload = json!({
                        "as_of": chrono::Utc::now().to_rfc3339(),
                        "claims": claim_views,
                        "terms": term_views,
                    });
                    let env = Envelope::success(EnvelopeKind::All, payload, vec![], vec![]);
                    emit_json(&env);
                }
            }
        }

        Command::Vocab { status, as_of } => {
            // Validate --as-of if supplied; emit structured error before opening the store.
            if let Some(ref raw) = as_of {
                if !is_valid_as_of(raw) {
                    emit_error_and_exit(
                        refusal(
                            "invalid-timestamp",
                            &format!(
                                "invalid --as-of value '{raw}'; expected RFC 3339 datetime or YYYY-MM-DD date"
                            ),
                            None,
                            vec![RemediationEntry {
                                command: "dont vocab --as-of 2026-01-01".to_string(),
                                description:
                                    "Use an ISO 8601 date or RFC 3339 datetime, e.g. 2026-01-01"
                                        .to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    );
                }
                // Historical snapshot queries are not yet implemented.
                emit_error_and_exit(
                    refusal(
                        "not-yet-implemented",
                        "historical snapshot queries (--as-of) are not yet implemented",
                        None,
                        vec![RemediationEntry {
                            command: "dont vocab --json".to_string(),
                            description: "Omit --as-of to query the current state".to_string(),
                        }],
                    ),
                    vec![],
                    1,
                );
            }
            let project = open_project_or_exit();
            let status_filter = match status {
                Some(raw) => match parse_claim_status_filter(&raw) {
                    Some(status) => Some(status),
                    None => emit_error_and_exit(
                        refusal(
                            "invalid-status",
                            &format!(
                                "unsupported term status '{raw}'; expected one of: unverified, verified, doubted, ignored"
                            ),
                            None,
                            vec![RemediationEntry {
                                command: "dont vocab --status unverified".to_string(),
                                description:
                                    "Use one of: unverified, verified, doubted, ignored, locked"
                                        .to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                },
                None => None,
            };
            let mut terms = match project.store.list_terms() {
                Ok(t) => t,
                Err(err) => handle_store_error(err, None),
            };
            if let Some(status_filter) = status_filter {
                terms.retain(|term| term.status == status_filter);
            }
            terms.sort_by(|a, b| {
                b.created_at
                    .cmp(&a.created_at)
                    .then_with(|| b.id.cmp(&a.id))
            });
            let views: Vec<Value> = terms
                .iter()
                .map(|term| build_term_view(term, &project.store))
                .collect();
            let env = Envelope::success(EnvelopeKind::TermList, views, vec![], vec![]);
            emit_json(&env);
        }

        Command::Trace { id } => {
            let project = open_project_or_exit();
            if let EntityId::Term(_) = EntityId::parse(&id) {
                match project.store.term_by_id(&id) {
                    Ok(Some(_)) => {
                        let payload = json!({
                            "entity_id": id,
                            "blockers": [],
                            "as_of": chrono::Utc::now().to_rfc3339(),
                        });
                        let env = Envelope::success(EnvelopeKind::Events, payload, vec![], vec![]);
                        emit_json(&env);
                    }
                    Ok(None) => emit_error_and_exit(
                        refusal(
                            "term-not-found",
                            &format!("no term with id {id}"),
                            Some(&id),
                            vec![RemediationEntry {
                                command: "dont list --kind=term".to_string(),
                                description: "List terms to find the correct id".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                    Err(err) => handle_store_error(err, Some(&id)),
                }
            } else {
                match project.store.claim_by_id(&id) {
                    Ok(Some(record)) => {
                        let blocker_paths: Vec<Value> = trace_claim(&record)
                            .into_iter()
                            .map(blocker_path_to_value)
                            .collect();
                        let payload = json!({
                            "entity_id": id,
                            "blockers": blocker_paths,
                            "as_of": chrono::Utc::now().to_rfc3339(),
                        });
                        let hints = if blocker_paths.is_empty() {
                            vec![]
                        } else {
                            vec![HintEntry {
                                command: format!("dont show {id}"),
                                description: "Inspect the entity details".to_string(),
                            }]
                        };
                        let env = Envelope::success(EnvelopeKind::Events, payload, vec![], hints);
                        emit_json(&env);
                    }
                    Ok(None) => emit_error_and_exit(
                        refusal(
                            "claim-not-found",
                            &format!("no claim with id {id}"),
                            Some(&id),
                            vec![RemediationEntry {
                                command: "dont list".to_string(),
                                description: "List all claims to find the correct id".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                    Err(err) => handle_store_error(err, Some(&id)),
                }
            }
        }

        Command::Stats {
            session,
            since,
            until,
        } => {
            validate_scope_flags(&session, &since, &until);
            let project = open_project_or_exit();
            let midnight = today_midnight_utc();
            let since_ref = since.as_deref().or(Some(midnight.as_str()));
            let until_ref = until.as_deref();
            let events = match project.store.all_events_in_scope(since_ref, until_ref) {
                Ok(e) => e,
                Err(err) => handle_store_error(err, None),
            };
            // Map event kind → canonical command name per spec.
            let event_to_verb: std::collections::HashMap<&str, &str> = [
                ("concluded", "conclude"),
                ("defined", "define"),
                ("trusted", "trust"),
                ("flagged", "flag"),
                ("dismissed", "flag"),
                ("locked", "lock"),
                ("ignored", "ignore"),
            ]
            .iter()
            .copied()
            .collect();
            let mut verb_counts: serde_json::Map<String, Value> = serde_json::Map::new();
            for ev in &events {
                let kind_str = ev.kind.as_str();
                if let Some(&verb) = event_to_verb.get(kind_str) {
                    let counter = verb_counts
                        .entry(verb.to_string())
                        .or_insert(Value::Number(0.into()));
                    if let Some(n) = counter.as_u64() {
                        *counter = Value::Number((n + 1).into());
                    }
                }
            }
            let idle_skill = verb_counts.is_empty();
            let claim_counts = match project.store.claim_counts_by_status() {
                Ok(c) => c,
                Err(err) => handle_store_error(err, None),
            };
            let total_claims: u64 = claim_counts.values().sum();
            let verified_claims = claim_counts.get("verified").copied().unwrap_or(0);
            let claim_verification_rate: Value = if total_claims == 0 {
                Value::Null
            } else {
                let rate = verified_claims as f64 / total_claims as f64;
                serde_json::to_value(rate).unwrap_or(Value::Null)
            };
            let caught_contradiction_count = match project
                .store
                .caught_contradiction_count(since_ref, until_ref)
            {
                Ok(c) => c,
                Err(err) => handle_store_error(err, None),
            };
            let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            let scope = build_scope_value(since_ref, until_ref, &now);
            let payload = json!({
                "scope": scope,
                "verb_counts": verb_counts,
                "dedup_refusal_count": 0u64,
                "claim_verification_rate": claim_verification_rate,
                "idle_skill": idle_skill,
                "caught_contradiction_count": caught_contradiction_count,
            });
            let env = Envelope::success(EnvelopeKind::Stats, payload, vec![], vec![]);
            emit_json(&env);
        }

        Command::Export {
            eval,
            session,
            since,
            until,
        } => {
            if !eval {
                emit_error_and_exit(
                    refusal(
                        "flag-required",
                        "export requires --eval flag",
                        None,
                        vec![RemediationEntry {
                            command: "dont export --eval --json".to_string(),
                            description: "Add --eval to export eval-harness data".to_string(),
                        }],
                    ),
                    vec![],
                    1,
                );
            }
            validate_scope_flags(&session, &since, &until);
            let project = open_project_or_exit();
            let midnight = today_midnight_utc();
            let since_ref = since.as_deref().or(Some(midnight.as_str()));
            let until_ref = until.as_deref();
            let exported_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            let scope = build_scope_value(since_ref, until_ref, &exported_at);
            // claims_by_status
            let claim_counts = match project.store.claim_counts_by_status() {
                Ok(c) => c,
                Err(err) => handle_store_error(err, None),
            };
            let claims_by_status: serde_json::Map<String, Value> = claim_counts
                .into_iter()
                .filter(|(_, v)| *v > 0)
                .map(|(k, v)| (k, Value::Number(v.into())))
                .collect();
            // events_by_kind
            let events = match project.store.all_events_in_scope(since_ref, until_ref) {
                Ok(e) => e,
                Err(err) => handle_store_error(err, None),
            };
            let mut events_by_kind: serde_json::Map<String, Value> = serde_json::Map::new();
            for ev in &events {
                let key = ev.kind.as_str().to_string();
                let counter = events_by_kind.entry(key).or_insert(Value::Number(0.into()));
                if let Some(n) = counter.as_u64() {
                    *counter = Value::Number((n + 1).into());
                }
            }
            // trust_events (Trusted and Flagged events with claim_id)
            let trust_rows = match project
                .store
                .trust_flag_events_with_claim_id(since_ref, until_ref)
            {
                Ok(r) => r,
                Err(err) => handle_store_error(err, None),
            };
            let trust_events: Vec<Value> = trust_rows
                .iter()
                .map(|r| {
                    let doubt = r.kind == "trusted";
                    let reason_excerpt = r
                        .note
                        .as_deref()
                        .unwrap_or("")
                        .chars()
                        .take(120)
                        .collect::<String>();
                    json!({
                        "event_id": r.event_id,
                        "target_claim_id": r.claim_id,
                        "doubt": doubt,
                        "reason_excerpt": reason_excerpt,
                        "timestamp": r.created_at,
                    })
                })
                .collect();
            let payload = json!({
                "exported_at": exported_at,
                "scope": scope,
                "claims_by_status": claims_by_status,
                "events_by_kind": events_by_kind,
                "trust_events": trust_events,
                "dedup_refusals": [],
            });
            let env = Envelope::success(EnvelopeKind::EvalExport, payload, vec![], vec![]);
            emit_json(&env);
        }

        Command::Completions { shell } => {
            let mut cmd = Cli::command();
            if !human_mode() {
                let mut script_buf = Vec::new();
                clap_complete::generate(shell, &mut cmd, "dont", &mut script_buf);
                let script = String::from_utf8_lossy(&script_buf);
                let shell_name = shell.to_string();
                let payload = json!({
                    "shell": shell_name,
                    "script": script.as_ref(),
                });
                emit_json(&Envelope::success(
                    EnvelopeKind::DontCompletions,
                    payload,
                    vec![],
                    vec![],
                ));
            } else {
                clap_complete::generate(shell, &mut cmd, "dont", &mut std::io::stdout());
            }
        }

        Command::Ground {
            statement,
            evidence,
            file,
            url,
            lines,
            anchor,
            excerpt,
        } => {
            // Pre-validate all inputs before writing any state. This ensures that
            // bad evidence or an empty statement leaves no partial claim behind.
            if statement == "-" {
                emit_error_and_exit(
                    refusal(
                        "stdin-not-supported",
                        "ground does not read entity IDs from stdin; provide the statement as an argument",
                        None,
                        vec![RemediationEntry {
                            command: "dont ground \"<claim text>\" --evidence <uri>".to_string(),
                            description: "Provide the statement directly as an argument"
                                .to_string(),
                        }],
                    ),
                    vec![],
                    2,
                );
            }
            if statement.trim().is_empty() {
                emit_error_and_exit(
                    refusal(
                        "empty-statement",
                        "statement: required; expected non-empty claim text",
                        None,
                        vec![RemediationEntry {
                            command: "dont ground \"<claim text>\" --evidence <uri>".to_string(),
                            description: "Provide a non-empty statement".to_string(),
                        }],
                    ),
                    vec![],
                    1,
                );
            }

            if let Some(err) = validate_claim_statement(&statement, "dont ground") {
                emit_error_and_exit(err, vec![], 1);
            }

            let evidence: Vec<String> = evidence
                .into_iter()
                .filter(|e| !e.trim().is_empty())
                .collect();

            if evidence.is_empty() && file.is_none() && url.is_none() {
                emit_error_and_exit(
                    refusal(
                        "no-evidence",
                        "--evidence: required; expected at least one URI, --file locator, or --url permalink",
                        None,
                        vec![RemediationEntry {
                            command: "dont ground \"<statement>\" --evidence <uri>".to_string(),
                            description: "Re-run with at least one evidence reference".to_string(),
                        }],
                    ),
                    vec![],
                    1,
                );
            }

            // Validate URI strings before opening the project so that a
            // malformed locator fails without leaving a partial claim behind.
            for uri in &evidence {
                if let Err(msg) = validate_evidence_uri(uri) {
                    emit_error_and_exit(
                        refusal(
                            "malformed-evidence-uri",
                            &msg,
                            None,
                            vec![RemediationEntry {
                                command: "dont ground \"<statement>\" --evidence <http://...>"
                                    .to_string(),
                                description:
                                    "Use a valid http:// or https:// URI as the evidence reference"
                                        .to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    );
                }
            }

            let project = open_project_or_exit();
            let project_root = project
                .dont_dir
                .parent()
                .unwrap_or(&project.dont_dir)
                .to_path_buf();

            let mut all_evidence: Vec<Value> = evidence.into_iter().map(Value::String).collect();
            if let Some(ref file_path) = file {
                all_evidence.push(resolve_file_locator(
                    file_path,
                    lines.as_deref(),
                    anchor.as_deref(),
                    excerpt.as_deref(),
                    &project_root,
                    None,
                    "dont ground \"<statement>\"",
                ));
            }
            if let Some(ref url_str) = url {
                all_evidence.push(resolve_url_locator(
                    url_str,
                    lines.as_deref(),
                    anchor.as_deref(),
                    excerpt.as_deref(),
                ));
            }

            // Write claim then immediately verify — both or neither.
            let conclude_result = match project.store.append_claim(&statement, &[], None) {
                Ok(r) => r,
                Err(err) => handle_store_error(err, None),
            };
            let claim_id = conclude_result.id.clone();

            apply_claim_transition(
                &project,
                &claim_id,
                model_flag,
                StoreEvent {
                    kind: StoreEventKind::Flagged,
                    note: None,
                    evidence: all_evidence,
                },
                "claim-not-found",
                "ground",
            );
        }

        Command::Atom { action } => {
            let project = open_project_or_exit();
            match action {
                AtomAction::Define { id, text } => {
                    if text.trim().is_empty() {
                        emit_error_and_exit(
                            refusal(
                                "empty-text",
                                "atom text must be non-empty",
                                Some(&id),
                                vec![RemediationEntry {
                                    command: format!("dont atom define {id} --text \"<atom>\""),
                                    description: "Provide a non-empty atom statement".to_string(),
                                }],
                            ),
                            vec![],
                            1,
                        );
                    }
                    let (result, _idx) = match project.store.define_atom(&id, &text) {
                        Ok(r) => r,
                        Err(StoreError::Malformed(ref msg)) if msg.contains("not found") => {
                            emit_error_and_exit(
                                refusal(
                                    "claim-not-found",
                                    &format!("claim {id} not found"),
                                    Some(&id),
                                    vec![RemediationEntry {
                                        command: "dont list".to_string(),
                                        description: "List claims to find the correct identifier"
                                            .to_string(),
                                    }],
                                ),
                                vec![],
                                1,
                            );
                        }
                        Err(err) => handle_store_error(err, Some(&id)),
                    };
                    let updated = match project.store.claim_by_id(&id) {
                        Ok(Some(r)) => r,
                        Ok(None) => handle_store_error(
                            StoreError::Malformed(format!("claim {id} vanished after atom define")),
                            Some(&id),
                        ),
                        Err(err) => handle_store_error(err, Some(&id)),
                    };
                    emit_claim_view(&updated, &result, &project.store);
                }

                AtomAction::Dismiss { id, idx, evidence } => {
                    if evidence.is_empty() {
                        emit_error_and_exit(
                            refusal(
                                "no-evidence",
                                "atom dismiss requires at least one --evidence item",
                                Some(&id),
                                vec![RemediationEntry {
                                    command: format!(
                                        "dont atom dismiss {id} {idx} --evidence <uri>"
                                    ),
                                    description: "Attach evidence for the atom verification"
                                        .to_string(),
                                }],
                            ),
                            vec![],
                            1,
                        );
                    }
                    let result = match project.store.dismiss_atom(&id, idx, &evidence) {
                        Ok(r) => r,
                        Err(StoreError::Malformed(ref msg)) if msg.contains("not found") => {
                            emit_error_and_exit(
                                refusal(
                                    "claim-not-found",
                                    &format!("claim {id} not found"),
                                    Some(&id),
                                    vec![RemediationEntry {
                                        command: "dont list".to_string(),
                                        description: "List claims to find the correct identifier"
                                            .to_string(),
                                    }],
                                ),
                                vec![],
                                1,
                            );
                        }
                        Err(StoreError::Malformed(ref msg)) if msg.contains("out of range") => {
                            emit_error_and_exit(
                                refusal(
                                    "atom-not-found",
                                    &format!("atom index {idx} does not exist on claim {id}"),
                                    Some(&id),
                                    vec![RemediationEntry {
                                        command: format!("dont show {id}"),
                                        description:
                                            "Inspect the claim to see available atom indices"
                                                .to_string(),
                                    }],
                                ),
                                vec![],
                                1,
                            );
                        }
                        Err(err) => handle_store_error(err, Some(&id)),
                    };
                    let updated = match project.store.claim_by_id(&id) {
                        Ok(Some(r)) => r,
                        Ok(None) => handle_store_error(
                            StoreError::Malformed(format!(
                                "claim {id} vanished after atom dismiss"
                            )),
                            Some(&id),
                        ),
                        Err(err) => handle_store_error(err, Some(&id)),
                    };
                    emit_claim_view(&updated, &result, &project.store);
                }
            }
        }

        Command::Hypothesis { action } => {
            let project = open_project_or_exit();
            match action {
                HypothesisAction::Add { id, text } => {
                    if text.trim().is_empty() {
                        emit_error_and_exit(
                            refusal(
                                "empty-text",
                                "hypothesis text must be non-empty",
                                Some(&id),
                                vec![RemediationEntry {
                                    command: format!(
                                        "dont hypothesis add {id} --text \"<hypothesis>\""
                                    ),
                                    description: "Provide a non-empty hypothesis statement"
                                        .to_string(),
                                }],
                            ),
                            vec![],
                            1,
                        );
                    }
                    let (result, _idx) = match project.store.add_hypothesis(&id, &text) {
                        Ok(r) => r,
                        Err(StoreError::Malformed(ref msg)) if msg.contains("not found") => {
                            emit_error_and_exit(
                                refusal(
                                    "claim-not-found",
                                    &format!("claim {id} not found"),
                                    Some(&id),
                                    vec![RemediationEntry {
                                        command: "dont list".to_string(),
                                        description: "List claims to find the correct identifier"
                                            .to_string(),
                                    }],
                                ),
                                vec![],
                                1,
                            );
                        }
                        Err(err) => handle_store_error(err, Some(&id)),
                    };
                    let updated = match project.store.claim_by_id(&id) {
                        Ok(Some(r)) => r,
                        Ok(None) => handle_store_error(
                            StoreError::Malformed(format!(
                                "claim {id} vanished after hypothesis add"
                            )),
                            Some(&id),
                        ),
                        Err(err) => handle_store_error(err, Some(&id)),
                    };
                    emit_claim_view(&updated, &result, &project.store);
                }

                HypothesisAction::Assess {
                    id,
                    idx,
                    supporting,
                    refuting,
                } => {
                    if supporting.is_empty() && refuting.is_empty() {
                        emit_error_and_exit(
                            refusal(
                                "no-assessment",
                                "hypothesis assess requires at least one --supporting or --refuting item",
                                Some(&id),
                                vec![RemediationEntry {
                                    command: format!(
                                        "dont hypothesis assess {id} {idx} --supporting <uri>"
                                    ),
                                    description:
                                        "Provide at least one supporting or refuting evidence reference"
                                            .to_string(),
                                }],
                            ),
                            vec![],
                            1,
                        );
                    }
                    let result =
                        match project
                            .store
                            .assess_hypothesis(&id, idx, &supporting, &refuting)
                        {
                            Ok(r) => r,
                            Err(StoreError::Malformed(ref msg)) if msg.contains("not found") => {
                                emit_error_and_exit(
                                    refusal(
                                        "claim-not-found",
                                        &format!("claim {id} not found"),
                                        Some(&id),
                                        vec![RemediationEntry {
                                            command: "dont list".to_string(),
                                            description:
                                                "List claims to find the correct identifier"
                                                    .to_string(),
                                        }],
                                    ),
                                    vec![],
                                    1,
                                );
                            }
                            Err(StoreError::Malformed(ref msg)) if msg.contains("out of range") => {
                                emit_error_and_exit(
                                    refusal(
                                        "hypothesis-not-found",
                                        &format!(
                                            "hypothesis index {idx} does not exist on claim {id}"
                                        ),
                                        Some(&id),
                                        vec![RemediationEntry {
                                        command: format!("dont show {id}"),
                                        description:
                                            "Inspect the claim to see available hypothesis indices"
                                                .to_string(),
                                    }],
                                    ),
                                    vec![],
                                    1,
                                );
                            }
                            Err(err) => handle_store_error(err, Some(&id)),
                        };
                    let updated = match project.store.claim_by_id(&id) {
                        Ok(Some(r)) => r,
                        Ok(None) => handle_store_error(
                            StoreError::Malformed(format!(
                                "claim {id} vanished after hypothesis assess"
                            )),
                            Some(&id),
                        ),
                        Err(err) => handle_store_error(err, Some(&id)),
                    };
                    emit_claim_view(&updated, &result, &project.store);
                }
            }
        }

        Command::Import { adapter, args } => {
            // --json may be captured as trailing var-arg if placed after the schema path
            if args.iter().any(|a| a == "--json") {
                HUMAN_MODE.with(|m| m.set(false));
            }
            let project = open_project_or_exit();
            let config = project.load_config();
            let adapter_cfg = config
                .import
                .adapters
                .get(&adapter)
                .cloned()
                .unwrap_or_default();
            if !adapter_cfg.is_enabled() {
                emit_error_and_exit(
                    refusal(
                        "adapter-disabled",
                        &format!("adapter '{adapter}' is disabled in this project's config.toml"),
                        None,
                        vec![RemediationEntry {
                            command: format!("[import.{adapter}]\nenabled = true"),
                            description: format!(
                                "Set enabled = true under [import.{adapter}] to re-enable this adapter"
                            ),
                        }],
                    ),
                    vec![],
                    1,
                );
            }
            if adapter == "linkml" {
                handle_linkml_import(&args, &project);
            } else {
                emit_error_and_exit(
                    refusal(
                        "not-implemented",
                        &format!(
                            "import adapter '{adapter}' is not yet implemented; only 'linkml' is currently supported"
                        ),
                        None,
                        vec![RemediationEntry {
                            command: "dont import linkml <schema.yaml>".to_string(),
                            description:
                                "Use the linkml adapter, the only currently supported adapter"
                                    .to_string(),
                        }],
                    ),
                    vec![],
                    1,
                );
            }
        }

        Command::Rules { action } => {
            let project = open_project_or_exit();
            let rules_dir = project.dont_dir.join("rules");
            let config = project.load_config();
            let engine = dont::rules::RuleEngine::new(
                rules_dir.clone(),
                config.rules,
                project.mode() == Some(ProjectMode::Strict),
            );

            match action {
                RulesAction::List => {
                    let mut rules: Vec<RuleInfo> = shipped_rule_names()
                        .map(|name| RuleInfo {
                            name: name.to_string(),
                            severity: severity_label(engine.severity(name)),
                            source: "shipped",
                        })
                        .collect();

                    if let Ok(entries) = std::fs::read_dir(&rules_dir) {
                        let mut custom: Vec<RuleInfo> = entries
                            .filter_map(|e| e.ok())
                            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("dl"))
                            .filter_map(|e| {
                                let stem = e.path().file_stem()?.to_str()?.to_string();
                                if shipped_rule_names().any(|n| n == stem) {
                                    return None;
                                }
                                Some(RuleInfo {
                                    severity: severity_label(engine.severity(&stem)),
                                    name: stem,
                                    source: "custom",
                                })
                            })
                            .collect();
                        custom.sort_by(|a, b| a.name.cmp(&b.name));
                        rules.extend(custom);
                    }

                    emit_json(&Envelope::success(
                        EnvelopeKind::RuleList,
                        rules,
                        vec![],
                        vec![],
                    ));
                }

                RulesAction::Show { name } => {
                    if shipped_rule_names().any(|n| n == name) {
                        let detail = RuleDetail {
                            name: name.clone(),
                            severity: severity_label(engine.severity(&name)),
                            source: "shipped",
                            datalog: None,
                        };
                        emit_json(&Envelope::success(
                            EnvelopeKind::Rule,
                            detail,
                            vec![],
                            vec![],
                        ));
                    } else {
                        let path = rules_dir.join(format!("{name}.dl"));
                        match std::fs::read_to_string(&path) {
                            Ok(src) => {
                                let detail = RuleDetail {
                                    name: name.clone(),
                                    severity: severity_label(engine.severity(&name)),
                                    source: "custom",
                                    datalog: Some(src),
                                };
                                emit_json(&Envelope::success(
                                    EnvelopeKind::Rule,
                                    detail,
                                    vec![],
                                    vec![],
                                ));
                            }
                            Err(_) => {
                                emit_error_and_exit(
                                    refusal(
                                        "rule-not-found",
                                        &format!("no rule named {name:?}"),
                                        None,
                                        vec![RemediationEntry {
                                            command: "dont rules list".to_string(),
                                            description: "List available rules".to_string(),
                                        }],
                                    ),
                                    vec![],
                                    1,
                                );
                            }
                        }
                    }
                }

                RulesAction::Add { file, force } => {
                    let src = match std::fs::read_to_string(&file) {
                        Ok(s) => s,
                        Err(e) => emit_error_and_exit(
                            refusal(
                                "rule-file-not-found",
                                &format!("cannot read {}: {e}", file.display()),
                                None,
                                vec![RemediationEntry {
                                    command: format!("ls {}", file.display()),
                                    description: "Verify the file path".to_string(),
                                }],
                            ),
                            vec![],
                            1,
                        ),
                    };

                    let rule_name = match file.file_stem().and_then(|s| s.to_str()) {
                        Some(name) => name,
                        None => emit_error_and_exit(
                            refusal(
                                "invalid-rule-filename",
                                &format!("cannot derive rule name from {:?}", file.display()),
                                None,
                                vec![RemediationEntry {
                                    command: "dont rules add <path/to/rule.dl>".to_string(),
                                    description: "Provide a valid .dl file path".to_string(),
                                }],
                            ),
                            vec![],
                            1,
                        ),
                    };

                    if shipped_rule_names().any(|n| n == rule_name) {
                        emit_error_and_exit(
                            refusal(
                                "cannot-shadow-shipped-rule",
                                &format!("rule name {rule_name:?} is reserved for a shipped rule"),
                                None,
                                vec![RemediationEntry {
                                    command: "dont rules list".to_string(),
                                    description: "See all shipped rules".to_string(),
                                }],
                            ),
                            vec![],
                            1,
                        );
                    }

                    if let Err(e) = project.store.run_rule_query(&src) {
                        emit_error_and_exit(
                            refusal(
                                "rule-compile-error",
                                &format!("rule {rule_name:?} failed to compile: {e}"),
                                None,
                                vec![RemediationEntry {
                                    command: format!("dont rules add {}", file.display()),
                                    description: "Fix the Datalog syntax and retry".to_string(),
                                }],
                            ),
                            vec![],
                            1,
                        );
                    }

                    let dest = rules_dir.join(format!("{rule_name}.dl"));

                    if !force && dest.exists() {
                        emit_error_and_exit(
                            refusal(
                                "rule-already-exists",
                                &format!("rule {rule_name:?} already exists"),
                                None,
                                vec![RemediationEntry {
                                    command: format!("dont rules add {} --force", file.display()),
                                    description: "Use --force to overwrite the existing rule"
                                        .to_string(),
                                }],
                            ),
                            vec![],
                            1,
                        );
                    }

                    if let Err(e) = std::fs::write(&dest, &src) {
                        emit_error_and_exit(
                            refusal(
                                "rule-write-error",
                                &format!("failed to write rule to {}: {e}", dest.display()),
                                None,
                                vec![RemediationEntry {
                                    command: format!("ls {}", rules_dir.display()),
                                    description: "Inspect the rules directory".to_string(),
                                }],
                            ),
                            vec![],
                            1,
                        );
                    }

                    emit_confirm_json(&Envelope::success(
                        EnvelopeKind::Empty,
                        serde_json::Value::Null,
                        vec![],
                        vec![
                            HintEntry {
                                command: "dont rules list".to_string(),
                                description: format!("Rule {rule_name:?} is now active"),
                            },
                            HintEntry {
                                command: format!(
                                    "# In .dont/config.toml under [rules]: warn = [\"{rule_name}\"]"
                                ),
                                description: format!(
                                    "Default severity is warn; add {rule_name:?} to [rules].strict only if you need unconditional refusals"
                                ),
                            },
                        ],
                    ));
                }

                RulesAction::Test { name } => {
                    let matches = match engine.evaluate_shipped(&project.store, &name) {
                        Some(Ok(m)) => m,
                        Some(Err(e)) => emit_error_and_exit(
                            refusal(
                                "rule-eval-error",
                                &format!("rule {name:?} failed to evaluate: {e}"),
                                None,
                                vec![RemediationEntry {
                                    command: format!("dont rules show {name}"),
                                    description: "Inspect the rule source".to_string(),
                                }],
                            ),
                            vec![],
                            1,
                        ),
                        None => match engine.evaluate(&project.store, &name) {
                            Ok(m) => m,
                            Err(RuleError::Io(ref e))
                                if e.kind() == std::io::ErrorKind::NotFound =>
                            {
                                emit_error_and_exit(
                                    refusal(
                                        "rule-not-found",
                                        &format!("no rule named {name:?}"),
                                        None,
                                        vec![RemediationEntry {
                                            command: "dont rules list".to_string(),
                                            description: "List available rules".to_string(),
                                        }],
                                    ),
                                    vec![],
                                    1,
                                )
                            }
                            Err(e) => emit_error_and_exit(
                                refusal(
                                    "rule-eval-error",
                                    &format!("rule {name:?} failed to evaluate: {e}"),
                                    None,
                                    vec![RemediationEntry {
                                        command: format!("dont rules show {name}"),
                                        description: "Inspect the rule source".to_string(),
                                    }],
                                ),
                                vec![],
                                1,
                            ),
                        },
                    };

                    let result = RuleTestResult {
                        severity: severity_label(engine.severity(&name)),
                        rule_name: name,
                        matches: matches
                            .into_iter()
                            .map(|m| RuleMatchView {
                                entity_id: m.entity_id,
                                detail: m.detail,
                            })
                            .collect(),
                    };
                    emit_json(&Envelope::success(
                        EnvelopeKind::RuleResult,
                        result,
                        vec![],
                        vec![],
                    ));
                }
            }
        }

        Command::Explain { rule } => {
            let project = open_project_or_exit();
            let rules_dir = project.dont_dir.join("rules");
            let config = project.load_config();
            let engine = dont::rules::RuleEngine::new(
                rules_dir,
                config.rules,
                project.mode() == Some(ProjectMode::Strict),
            );

            if let Some(prose) = dont::rules::explain(&rule) {
                let severity = severity_label(engine.severity(&rule));
                let payload = json!({
                    "rule_name": rule,
                    "severity": severity,
                    "source": "shipped",
                    "explanation": prose,
                });
                if human_mode() {
                    println!("{}", prose.trim());
                } else {
                    emit_json(&Envelope::success(
                        EnvelopeKind::DontExplain,
                        payload,
                        vec![],
                        vec![],
                    ));
                }
            } else {
                emit_error_and_exit(
                    refusal(
                        "rule-not-found",
                        &format!("no rule named '{rule}'"),
                        None,
                        vec![RemediationEntry {
                            command: "dont rules list".to_string(),
                            description: "List available rules".to_string(),
                        }],
                    ),
                    vec![],
                    1,
                );
            }
        }
        // Dismiss and Lock are normalized to Flag and Forget above; these arms
        // are unreachable but required for exhaustiveness.
        Command::Dismiss { .. }
        | Command::Lock { .. }
        | Command::Record { .. }
        | Command::Challenge { .. } => {
            unreachable!("aliases and cross-vocab commands are normalized before this match")
        }

        // Help: agent-addressed help, tutorial, and how-to guides (dont-nolt).
        Command::Help {
            command: cmd_name,
            tutorial,
            topics,
            howto,
        } => {
            if tutorial {
                let text: String = HELP_TUTORIAL.iter().map(|s| s.to_string()).collect();
                print!("{text}");
            } else if topics {
                print!("tutorial");
                for (name, desc) in HOWTO_TOPICS {
                    print!("\nhowto:{name}  -- {desc}");
                }
                println!();
            } else if let Some(topic) = howto {
                match howto_content(&topic) {
                    Some(guide) => print!("{guide}"),
                    None => {
                        emit_error_and_exit(
                            refusal(
                                "not-found",
                                &format!("no how-to guide for topic '{topic}'"),
                                None,
                                vec![RemediationEntry {
                                    command: "dont help --topics".to_string(),
                                    description: "List available how-to topics".to_string(),
                                }],
                            ),
                            vec![],
                            1,
                        );
                    }
                }
            } else if let Some(name) = cmd_name {
                let mut app = Cli::command();
                if let Some(sub) = app.find_subcommand_mut(&name) {
                    let _ = sub.print_help();
                } else {
                    emit_error_and_exit(
                        refusal(
                            "not-found",
                            &format!("no command named '{name}'"),
                            None,
                            vec![RemediationEntry {
                                command: "dont help".to_string(),
                                description: "List available commands".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    );
                }
            } else {
                // Bare `dont help` — list subcommands, then tutorial/how-to entry points.
                // Spec dont-cli-surface: "bare help lists available commands"
                let mut app = Cli::command();
                println!("Commands:");
                for sub in app.get_subcommands_mut() {
                    let name = sub.get_name().to_string();
                    let about = sub.get_about().map(|s| s.to_string()).unwrap_or_default();
                    println!("  {name:<20} {about}");
                }
                println!();
                println!("Use `dont help --tutorial` for the first-session walkthrough.");
                println!("Use `dont help --topics` to list all how-to guides.");
                println!("Use `dont help --howto <topic>` to read a specific guide.");
                println!();
                print!("tutorial  -- First-session walkthrough");
                for (name, desc) in HOWTO_TOPICS {
                    print!("\nhowto:{name}  -- {desc}");
                }
                println!();
            }
        }
    }
}

#[cfg(test)]
mod parse_line_span_tests {
    use super::parse_line_span;

    // Valid inputs
    #[test]
    fn single_line_number() {
        assert_eq!(parse_line_span("42"), Ok((42, 42)));
    }

    #[test]
    fn parse_line_span_range_string_returns_start_and_end() {
        assert_eq!(parse_line_span("10-18"), Ok((10, 18)));
    }

    #[test]
    fn single_line_one() {
        assert_eq!(parse_line_span("1"), Ok((1, 1)));
    }

    #[test]
    fn range_same_start_end() {
        assert_eq!(parse_line_span("5-5"), Ok((5, 5)));
    }

    #[test]
    fn leading_trailing_whitespace_trimmed() {
        assert_eq!(parse_line_span(" 10 "), Ok((10, 10)));
    }

    // Invalid: zero-based lines
    #[test]
    fn zero_single_is_err() {
        assert!(parse_line_span("0").is_err());
    }

    #[test]
    fn zero_start_range_is_err() {
        assert!(parse_line_span("0-5").is_err());
    }

    #[test]
    fn zero_end_range_is_err() {
        assert!(parse_line_span("1-0").is_err());
    }

    // Invalid: start > end
    #[test]
    fn start_greater_than_end_is_err() {
        assert!(parse_line_span("10-5").is_err());
    }

    // Invalid: non-numeric content
    #[test]
    fn empty_string_is_err() {
        assert!(parse_line_span("").is_err());
    }

    #[test]
    fn alphabetic_is_err() {
        assert!(parse_line_span("abc").is_err());
    }

    #[test]
    fn alphanumeric_range_is_err() {
        assert!(parse_line_span("10-x").is_err());
    }

    #[test]
    fn decimal_is_err() {
        assert!(parse_line_span("10.5").is_err());
    }

    // Invalid: negative numbers
    #[test]
    fn negative_single_is_err() {
        // "-5" → split_once('-') → ("", "5") → "" parse fails
        assert!(parse_line_span("-5").is_err());
    }

    #[test]
    fn negative_end_is_err() {
        // "10--2" → split_once('-') → ("10", "-2") → "-2" parse fails
        assert!(parse_line_span("10--2").is_err());
    }

    // Invalid: structural
    #[test]
    fn dash_only_is_err() {
        assert!(parse_line_span("-").is_err());
    }

    #[test]
    fn missing_end_is_err() {
        assert!(parse_line_span("1-").is_err());
    }

    #[test]
    fn missing_start_is_err() {
        assert!(parse_line_span("-1").is_err());
    }

    #[test]
    fn triple_segment_is_err() {
        // "1-2-3" → split_once gives ("1", "2-3") → "2-3".parse::<u32>() fails
        assert!(parse_line_span("1-2-3").is_err());
    }

    // Invalid: overflow
    #[test]
    fn u32_overflow_is_err() {
        assert!(parse_line_span("9999999999").is_err());
    }

    // Invalid: whitespace only
    #[test]
    fn whitespace_only_is_err() {
        assert!(parse_line_span("   ").is_err());
    }

    // Error message quality
    #[test]
    fn invalid_number_error_contains_input() {
        let err = parse_line_span("abc").unwrap_err();
        assert!(
            err.contains("abc"),
            "error should mention the bad input: {err}"
        );
    }

    #[test]
    fn invalid_range_error_contains_input() {
        let err = parse_line_span("10-x").unwrap_err();
        assert!(
            err.contains("10-x"),
            "error should mention the bad input: {err}"
        );
    }

    // Boundary values
    #[test]
    fn valid_u32_max() {
        // u32::MAX should parse as a valid (if extreme) line number
        assert_eq!(parse_line_span("4294967295"), Ok((4294967295, 4294967295)));
    }

    #[test]
    fn leading_zeros_parsed_as_decimal() {
        // Rust's u32 parse treats "01" as 1, not an error — document this behavior
        assert_eq!(parse_line_span("01"), Ok((1, 1)));
        assert_eq!(parse_line_span("01-02"), Ok((1, 2)));
    }

    #[test]
    fn whitespace_around_dash_accepted() {
        // split_once trims each segment; "10 - 20" → ("10 ", " 20") → (10, 20)
        assert_eq!(parse_line_span("10 - 20"), Ok((10, 20)));
    }
}

#[cfg(test)]
mod label_validation_tests {
    use super::*;

    // --- label_has_indefinite_article ---

    #[test]
    fn article_alone_a_is_invalid() {
        assert!(!label_has_indefinite_article("a"));
    }

    #[test]
    fn article_alone_an_is_invalid() {
        assert!(!label_has_indefinite_article("an"));
    }

    #[test]
    fn article_with_word_after_is_valid() {
        assert!(label_has_indefinite_article("  a  noun  "));
    }

    #[test]
    fn article_mid_sentence_not_at_start_is_invalid() {
        assert!(!label_has_indefinite_article("the a word"));
    }

    #[test]
    fn uppercase_a_is_valid() {
        assert!(label_has_indefinite_article("A thing"));
    }

    #[test]
    fn uppercase_an_is_valid() {
        assert!(label_has_indefinite_article("An event"));
    }

    // --- label_ends_with_sentence_punctuation ---

    #[test]
    fn trailing_whitespace_after_period_is_flagged() {
        assert!(label_ends_with_sentence_punctuation("a thing.   "));
    }

    #[test]
    fn trailing_comma_is_not_flagged() {
        assert!(!label_ends_with_sentence_punctuation("a thing,"));
    }

    #[test]
    fn no_punctuation_clean() {
        assert!(!label_ends_with_sentence_punctuation("a clean label"));
    }

    // --- label_compound_undeclared ---

    #[test]
    fn pair_with_empty_parens_is_undeclared() {
        assert!(label_compound_undeclared("a pair ()"));
    }

    #[test]
    fn pair_with_trailing_comma_arity_one_is_undeclared() {
        // "a pair (x,)" → filtered count = 1, required = 2 → mismatch
        assert!(label_compound_undeclared("a pair (x,)"));
    }

    #[test]
    fn sequence_with_empty_parens_is_undeclared() {
        assert!(label_compound_undeclared("a sequence ()"));
    }

    #[test]
    fn list_of_with_one_var_is_declared() {
        assert!(!label_compound_undeclared("a list of (x)"));
    }

    #[test]
    fn pair_with_correct_arity_is_declared() {
        assert!(!label_compound_undeclared("a pair (x, y)"));
    }

    #[test]
    fn triple_with_correct_arity_is_declared() {
        assert!(!label_compound_undeclared("a triple (a, b, c)"));
    }

    #[test]
    fn set_of_with_multiple_vars_is_declared() {
        assert!(!label_compound_undeclared("a set of (x, y, z)"));
    }

    // --- label_contains_sentence_verb ---

    #[test]
    fn verb_after_close_paren_before_where_is_flagged() {
        assert!(label_contains_sentence_verb("a pair (x, y) is valid"));
    }

    #[test]
    fn verb_in_where_clause_only_is_not_flagged() {
        assert!(!label_contains_sentence_verb(
            "a pair (x, y) where x is an integer"
        ));
    }

    #[test]
    fn verb_before_open_paren_is_flagged() {
        assert!(label_contains_sentence_verb("a thing has (x)"));
    }

    #[test]
    fn no_verb_no_parens_is_clean() {
        assert!(!label_contains_sentence_verb("a simple noun phrase"));
    }

    // --- best_article_for ---

    #[test]
    fn empty_string_returns_a_no_panic() {
        assert_eq!(best_article_for(""), "a");
    }

    #[test]
    fn vowel_start_returns_an() {
        assert_eq!(best_article_for("umbrella"), "an");
    }

    #[test]
    fn consonant_start_returns_a() {
        assert_eq!(best_article_for("book"), "a");
    }

    #[test]
    fn y_consonant_sound_returns_a() {
        assert_eq!(best_article_for("you"), "a");
    }
}
