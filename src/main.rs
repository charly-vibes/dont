use std::cell::Cell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use dont::envelope::{
    CLI_VERSION, ENVELOPE_VERSION, Envelope, ErrorResult, HintEntry, RemediationEntry,
    UnmetClause, Warning, set_author,
};
use dont::model::{
    Status, flag as model_flag, ignore as model_ignore, lock as model_lock,
    reopen as model_reopen, trust as model_trust, undoubt as model_undoubt,
};
use dont::config::{DefineShapeConfig, TermNonfunctionalConfig};
use dont::project::{Project, ProjectError, ProjectMode};
use dont::rules::{RuleError, SHIPPED_RULES};
use dont::store::{
    AppendResult, ClaimRecord, EntityResolution, EventRecord, HypothesisRecord, Store, StoreError,
    StoreEvent, StoreEventKind, StoreStatus, TermRecord,
};

thread_local! {
    static HUMAN_MODE: Cell<bool> = const { Cell::new(false) };
    static PLAIN_MODE: Cell<bool> = const { Cell::new(false) };
}

fn human_mode() -> bool {
    HUMAN_MODE.with(|m| m.get())
}

fn color_enabled() -> bool {
    use std::io::IsTerminal;
    if PLAIN_MODE.with(|m| m.get()) {
        return false;
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
#[command(about = "Epistemic forcing-function CLI for grounded claims")]
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
    #[arg(long, global = true)]
    plain: bool,

    /// Author identifier for this invocation. Overrides $DONT_AUTHOR.
    #[arg(long, short = 'a', global = true)]
    author: Option<String>,

    /// Bypass harness detection; behave as if DONT_DIRECT=1.
    #[arg(long, global = true)]
    direct: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Initialize dont state in the current project.
    Init {
        /// Start the project in strict mode instead of permissive mode.
        #[arg(long)]
        strict: bool,
    },

    /// Introduce an unverified claim.
    Conclude {
        /// Claim statement text.
        statement: String,

        /// CURIE of a term this claim depends on. May be repeated.
        #[arg(long)]
        depends_on: Vec<String>,
    },

    /// Introduce an unverified coined term.
    Define {
        /// Term CURIE, e.g. WB:P001.
        curie: Option<String>,

        /// Prose definition for the term.
        #[arg(long)]
        doc: Option<String>,

        /// SK11 type-text: singular indefinite noun phrase for the term box in an olog.
        #[arg(long)]
        label: Option<String>,
    },

    /// Register explicit doubt about a claim. Read as 'dont trust' = 'do not trust it'.
    Trust {
        /// Claim identifier.
        id: String,

        /// Reason for doubt (required).
        #[arg(long, short)]
        reason: Option<String>,
    },

    /// Verify a claim with evidence. Read as 'dont flag' = 'do not flag it as a concern'.
    Flag {
        /// Claim identifier.
        id: String,

        /// Evidence URI or reference.
        #[arg(long, short)]
        evidence: Vec<String>,

        /// Repository-relative file path for a structured evidence locator.
        #[arg(long)]
        file: Option<String>,

        /// Line span within the file, e.g. "10-18" or "42".
        #[arg(long)]
        lines: Option<String>,

        /// Named anchor within the file.
        #[arg(long)]
        anchor: Option<String>,

        /// Captured excerpt from the referenced source for later audit.
        #[arg(long)]
        excerpt: Option<String>,
    },

    /// Retract doubt on a doubted entity, returning it to unverified. Use 'reopen' for ignored entities.
    Undoubt {
        /// Entity identifier (claim:... or term:...).
        id: String,
    },

    /// Permanently preserve a verified claim when the lockable gate is met. Read as 'dont forget' = 'do not forget it'.
    Forget {
        /// Claim identifier.
        id: String,
    },

    /// Restore an ignored claim or term to unverified status.
    Reopen {
        /// Entity identifier (claim:... or term:...).
        id: String,
    },

    /// Move a claim or term to ignored state.
    Ignore {
        /// Entity identifier (claim:... or term:...).
        id: String,

        /// Substantive reason for ignoring (required; hedge-only reasons are refused).
        #[arg(long, short)]
        reason: Option<String>,
    },

    /// Show a claim or term.
    Show {
        /// Claim or term identifier (claim:ID, term:ID, or CURIE like WB:P001).
        id: String,
    },

    /// Explain why a claim or term has its current status.
    Why {
        /// Claim or term identifier (claim:ID, term:ID, or CURIE like WB:P001).
        id: String,
    },

    /// Check liveness of attached evidence references without changing status.
    VerifyEvidence {
        /// Entity identifier (claim:... or term:...).
        id: String,

        /// Per-reference timeout in seconds.
        #[arg(long)]
        timeout_seconds: Option<u64>,
    },

    /// Return session-start orientation and project state summary.
    Prime,

    /// List entities.
    List {
        /// Filter entities by status.
        #[arg(long)]
        status: Option<String>,

        /// Choose whether to list claims or terms.
        #[arg(long)]
        kind: Option<String>,
    },

    /// Explain the blocker-path for a claim or term.
    Trace {
        /// Entity identifier (claim:... or term:...).
        id: String,
    },

    /// Generate shell completion scripts.
    Completions {
        /// Shell to generate completions for (bash, zsh, fish, powershell, elvish).
        shell: Shell,
    },

    /// Atomically ground a claim with its supporting evidence.
    Ground {
        /// Claim statement text.
        statement: String,

        /// Evidence URI or reference.
        #[arg(long, short)]
        evidence: Vec<String>,

        /// Repository-relative file path for a structured evidence locator.
        #[arg(long)]
        file: Option<String>,

        /// Line span within the file, e.g. "10-18" or "42".
        #[arg(long)]
        lines: Option<String>,

        /// Named anchor within the file.
        #[arg(long)]
        anchor: Option<String>,

        /// Captured excerpt from the referenced source.
        #[arg(long)]
        excerpt: Option<String>,
    },

    /// Manage independently checkable atoms for a claim.
    Atom {
        #[command(subcommand)]
        action: AtomAction,
    },

    /// Manage competing hypotheses for a claim.
    Hypothesis {
        #[command(subcommand)]
        action: HypothesisAction,
    },

    /// Import terms from an external ontology adapter.
    Import {
        /// Adapter name (obo, ols, wikidata, openalex, bioregistry, jsonld, ttl, linkml).
        adapter: String,

        /// Adapter-specific arguments.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Manage and inspect project rules.
    Rules {
        #[command(subcommand)]
        action: RulesAction,
    },

    /// Show the prose explanation for a rule: what it checks, why it matters, and how to satisfy it.
    Explain {
        /// Rule name (e.g. ungrounded, lockable, correlated-error).
        rule: String,
    },
}

#[derive(Debug, Subcommand)]
enum AtomAction {
    /// Add an independently checkable atom to a claim.
    Define {
        /// Claim identifier.
        id: String,

        /// Atom text.
        #[arg(long)]
        text: String,
    },

    /// Mark an atom verified with evidence.
    Dismiss {
        /// Claim identifier.
        id: String,

        /// Atom index (0-based).
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
        id: String,

        /// Hypothesis text.
        #[arg(long)]
        text: String,
    },

    /// Assess a hypothesis with supporting or refuting evidence.
    Assess {
        /// Claim identifier.
        id: String,

        /// Hypothesis index (0-based).
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
        name: String,
    },

    /// Install a project-specific rule from a .dl file.
    Add {
        /// Path to the .dl file.
        file: PathBuf,
        /// Overwrite an existing rule with the same name.
        #[arg(long)]
        force: bool,
    },

    /// Dry-run a rule against the current store without modifying state.
    Test {
        /// Rule name.
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

fn contains_hedge(reason: &str, extra: &[String]) -> bool {
    let lower = reason.to_lowercase();
    DEFAULT_HEDGES.iter().any(|h| lower.contains(h))
        || extra.iter().any(|h| lower.contains(h.as_str()))
}

fn cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
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
    } else {
        println!("{}", serde_json::to_string(envelope).unwrap());
    }
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
                        command: format!("dont define {} --doc \"<definition>\"", suggest_alternative_curie(&curie)),
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
                description: "Inspect the project state directory for obvious corruption or missing files".to_string(),
            },
            RemediationEntry {
                command: "https://github.com/charly-vibes/dont/issues".to_string(),
                description: "Report the issue if the project state looks intact".to_string(),
            },
        ],
    };
    emit_error_no_exit(err_result, vec![], 4)
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
        "terms" => format_terms_list(data),
        "prime" => format_prime(data),
        "trace" => format_trace(data),
        "evidence_check" => format_evidence_check(data),
        _ => format!("ok  {kind}"),
    }
}

fn format_claims_list(data: &Value) -> String {
    let items = match data.as_array() {
        Some(arr) => arr,
        None => return "(no claims)".to_string(),
    };
    if items.is_empty() {
        return "(no claims)".to_string();
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
        None => return "(no terms)".to_string(),
    };
    if items.is_empty() {
        return "(no terms)".to_string();
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
                    format!("    {s}")
                } else if ev.get("kind").and_then(Value::as_str) == Some("repo-file") {
                    let path = ev["path"].as_str().unwrap_or("?");
                    format!("    repo:{path}")
                } else {
                    format!("    {ev}")
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
                .map(|v| v.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", "))
                .unwrap_or_default();
            let ref_str = refuting
                .map(|v| v.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", "))
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
    let mut out = format!("{id}  {curie}\n  status:      {colored_status}\n  definition:  {definition}");
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
            let check = if outcome == "ok" { "ok" } else { "fail" };
            out.push_str(&format!("\n  [{check}] {uri}"));
            if let Some(detail) = r["detail"].as_str() {
                out.push_str(&format!(" ({detail})"));
            }
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
            3,
        ),
        ProjectError::ConfigMissing(msg) => ("config-missing".to_string(), msg.clone(), 3),
        ProjectError::LayoutInvalid(_) => ("layout-invalid".to_string(), err.to_string(), 3),
        ProjectError::Store(_) => ("internal".to_string(), err.to_string(), 4),
        ProjectError::Io(_) => ("internal".to_string(), err.to_string(), 4),
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
        ProjectError::LayoutInvalid(_) => vec![RemediationEntry {
            command: "dont init".to_string(),
            description: "Run dont init to repair the missing project directories".to_string(),
        }],
        _ => vec![
            RemediationEntry {
                command: "ls ${DONT_DIR:-.dont}".to_string(),
                description: "Inspect the project state directory for obvious corruption or missing files".to_string(),
            },
            RemediationEntry {
                command: "https://github.com/charly-vibes/dont/issues".to_string(),
                description: "Report the issue if the project state looks intact".to_string(),
            },
        ],
    }
}

fn open_project_or_exit() -> Project {
    match Project::open(&cwd()) {
        Ok(p) => {
            p.check_and_record_mode_change();
            p
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

fn store_status_from_model(s: Status) -> StoreStatus {
    match s {
        Status::Unverified => StoreStatus::Unverified,
        Status::Verified => StoreStatus::Verified,
        Status::Doubted => StoreStatus::Doubted,
        Status::Ignored => StoreStatus::Ignored,
        Status::Locked => StoreStatus::Locked,
    }
}

fn model_status_from_store(s: StoreStatus) -> Status {
    match s {
        StoreStatus::Unverified => Status::Unverified,
        StoreStatus::Verified => Status::Verified,
        StoreStatus::Doubted => Status::Doubted,
        StoreStatus::Ignored => Status::Ignored,
        StoreStatus::Locked => Status::Locked,
    }
}

fn parse_claim_status_filter(status: &str) -> Option<StoreStatus> {
    match status.trim().to_ascii_lowercase().as_str() {
        "unverified" => Some(StoreStatus::Unverified),
        "verified" => Some(StoreStatus::Verified),
        "doubted" => Some(StoreStatus::Doubted),
        "ignored" => Some(StoreStatus::Ignored),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListKind {
    Claims,
    Terms,
}

fn parse_list_kind(kind: &str) -> Option<ListKind> {
    match kind.trim().to_ascii_lowercase().as_str() {
        "claims" => Some(ListKind::Claims),
        "terms" => Some(ListKind::Terms),
        _ => None,
    }
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

fn build_term_view(record: &TermRecord, store: &Store) -> Value {
    let project_root = project_root_from_store(store);
    let evidence = project_evidence(collect_term_evidence(record), &project_root);
    let updated_at = record
        .events
        .iter()
        .map(|e| &e.created_at)
        .max()
        .cloned()
        .unwrap_or_else(|| record.created_at.clone());
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
        "updated_at": updated_at,
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
    json!({
        "entity": entity,
        "history": build_event_history(&record.events),
        "applicable_rules": entity["applicable_rules"].clone(),
        "remediation": [],
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
    if let Some(path) = v.as_object()
        .filter(|obj| obj.get("kind").and_then(Value::as_str) == Some("repo-file"))
        .and_then(|obj| obj.get("path"))
        .and_then(Value::as_str)
    {
        return format!("repo-file:{path}");
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
        return Err("absolute paths are not allowed as repository locators; use a project-relative path");
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
            Component::ParentDir => { full.pop(); }
            Component::Normal(c) => full.push(c),
            Component::CurDir => {}
            _ => {}
        }
    }
    if let (Ok(canonical_root), Ok(canonical_target)) = (project_root.canonicalize(), full.canonicalize())
        && !canonical_target.starts_with(&canonical_root)
    {
        return Err("path escapes project root");
    }
    Ok(full.strip_prefix(project_root).unwrap_or(&full).to_path_buf())
}

/// Parse a line span string like "10-18" or "42" into (start, end).
fn parse_line_span(s: &str) -> Result<(u32, u32), String> {
    if let Some((a, b)) = s.split_once('-') {
        let start: u32 = a.trim().parse().map_err(|_| format!("invalid line span: {s}"))?;
        let end: u32 = b.trim().parse().map_err(|_| format!("invalid line span: {s}"))?;
        if start == 0 || end == 0 {
            return Err("line spans are one-based; line 0 is invalid".to_string());
        }
        if start > end {
            return Err(format!("line span start {start} is greater than end {end}"));
        }
        Ok((start, end))
    } else {
        let line: u32 = s.trim().parse().map_err(|_| format!("invalid line number: {s}"))?;
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
    if record.depends_on.is_empty() {
        return derived;
    }

    for dep in &record.depends_on {
        let lookup = if dep.starts_with("term:") {
            store.term_by_id(dep)
        } else {
            store.term_by_curie(dep)
        };
        match lookup {
            Ok(Some(term)) => match term.status {
                StoreStatus::Verified => {}
                StoreStatus::Ignored | StoreStatus::Locked => {
                    if !derived.iter().any(|d| d == "compromised-support") {
                        derived.push("compromised-support".to_string());
                    }
                }
                StoreStatus::Unverified | StoreStatus::Doubted => {
                    if !derived.iter().any(|d| d == "stale") {
                        derived.push("stale".to_string());
                    }
                }
            },
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
    term_result: Result<Option<TermRecord>, StoreError>,
) -> Option<BlockerPath> {
    let path = vec![start_id.to_string(), dep.to_string()];
    match term_result {
        Ok(Some(term)) => {
            let (kind, remediation) = match term.status {
                StoreStatus::Unverified | StoreStatus::Doubted => (
                    "stale",
                    vec![RemediationEntry {
                        command: format!("dont dismiss {}", term.id),
                        description: format!("Verify the blocking term {}", term.id),
                    }],
                ),
                StoreStatus::Ignored | StoreStatus::Locked => (
                    "compromised-support",
                    vec![RemediationEntry {
                        command: format!("dont show {}", term.id),
                        description: format!(
                            "Inspect the compromised supporting term {}",
                            term.id
                        ),
                    }],
                ),
                StoreStatus::Verified => return None,
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
            project.store.term_by_id(dep)
        } else {
            project.store.term_by_curie(dep)
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
    let hypothesis_count = assessed_hypothesis_count(&record.hypotheses);
    if hypothesis_count < 3 {
        unmet.push(UnmetClause {
            clause: format!("needs >=3 assessed hypotheses; has {hypothesis_count}"),
            fix: "record and assess at least three competing hypotheses before locking"
                .to_string(),
        });
    }

    let evidence_count = independent_evidence_count(record);
    if evidence_count < 2 {
        unmet.push(UnmetClause {
            clause: format!("needs >=2 independent supporting evidence items; has {evidence_count}"),
            fix: "attach evidence from at least two independent sources before locking"
                .to_string(),
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
            fix: "resolve dependency integrity issues before dismissing this claim"
                .to_string(),
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

    let status = std::process::Command::new("git")
        .args(["-C", &root, "status", "--porcelain", &rel])
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
        "claim",
        payload,
        vec![],
        vec![HintEntry {
            command: format!("dont show {}", record.id),
            description: "Inspect the updated claim".to_string(),
        }],
        Some(result.tx as u64),
    );
    emit_json(&env);
}

fn emit_term_view(record: &TermRecord, result: &AppendResult, store: &Store, warnings: Vec<Warning>) {
    let payload = build_term_view(record, store);
    let env = Envelope::success_with_tx(
        "term",
        payload,
        warnings,
        vec![HintEntry {
            command: format!("dont show {}", record.id),
            description: "Inspect the new term".to_string(),
        }],
        Some(result.tx as u64),
    );
    emit_json(&env);
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
    // Detect removed "lock" subcommand and suggest "forget" before clap parses.
    let first_subcmd = std::env::args()
        .skip(1)
        .find(|a| !a.starts_with('-'));
    if first_subcmd.as_deref() == Some("lock") {
        eprintln!("error: unknown command 'lock'. Did you mean 'dont forget'?");
        process::exit(1);
    }

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
    if cli.plain && !cli.json {
        PLAIN_MODE.with(|m| m.set(true));
    }

    // --version [--json]
    if cli.version {
        if cli.json {
            let env = Envelope::success(
                "version",
                json!({
                    "cli_version": CLI_VERSION,
                    "envelope_version": ENVELOPE_VERSION,
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
                        "empty",
                        json!({ "mode": mode.as_str() }),
                        vec![],
                        vec![HintEntry {
                            command: "dont conclude \"claim text\"".to_string(),
                            description: "Introduce your first claim".to_string(),
                        }],
                    );
                    emit_json(&env);
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
                            description: "Provide the statement directly as an argument".to_string(),
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
                        "conclude requires a non-empty claim statement",
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
                    match project.store.term_by_curie(dep) {
                        Ok(Some(term)) => resolved_depends_on.push(term.id),
                        Ok(None) => unresolved.push(dep.clone()),
                        Err(err) => handle_store_error(err, None),
                    }
                }
            }

            let is_strict = project.mode() == "strict";
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
            match project.store.append_claim(&statement, &all_depends_on) {
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
                        "claim",
                        payload,
                        warnings,
                        vec![HintEntry {
                            command: format!("dont show {}", result.id),
                            description: "Inspect the new claim".to_string(),
                        }],
                        Some(result.tx as u64),
                    );
                    emit_json(&env);
                }
                Err(err) => handle_store_error(err, None),
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
                        "define requires --doc with a non-empty prose definition",
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

        Command::Trust { id, reason } => {
            let reason = match reason {
                None => {
                    emit_error_and_exit(
                        refusal(
                            "reason-required",
                            "trust requires --reason: state what specific grounds you have for doubt",
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
            if id.starts_with("term:") {
                let record = match project.store.term_by_id(&id) {
                    Ok(Some(r)) => r,
                    Ok(None) => emit_error_and_exit(
                        refusal(
                            "term-not-found",
                            &format!("no term with id {id}"),
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
                };

                let current = model_status_from_store(record.status);
                match model_trust(current) {
                    Err(transition_err) => {
                        emit_error_and_exit(
                            refusal(
                                &transition_err.code,
                                &transition_err.message,
                                Some(&id),
                                vec![RemediationEntry {
                                    command: format!("dont show {id}"),
                                    description: "Inspect the current term status".to_string(),
                                }],
                            ),
                            vec![],
                            1,
                        );
                    }
                    Ok(new_model_status) => {
                        let event = StoreEvent {
                            kind: StoreEventKind::Trusted,
                            note: Some(reason),
                            evidence: vec![],
                        };
                        let result = match project.store.append_term_status_change(
                            &id,
                            store_status_from_model(current),
                            store_status_from_model(new_model_status),
                            event,
                        ) {
                            Ok(r) => r,
                            Err(err) => handle_store_error(err, Some(&id)),
                        };
                        let updated = match project.store.term_by_id(&id) {
                            Ok(Some(r)) => r,
                            Ok(None) => handle_store_error(
                                StoreError::Malformed(format!("term {id} vanished after trust")),
                                Some(&id),
                            ),
                            Err(err) => handle_store_error(err, Some(&id)),
                        };
                        emit_term_view(&updated, &result, &project.store, vec![]);
                        return;
                    }
                }
            }

            let record = match project.store.claim_by_id(&id) {
                Ok(Some(r)) => r,
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
            };

            let current = model_status_from_store(record.status);
            match model_trust(current) {
                Err(transition_err) => {
                    emit_error_and_exit(
                        refusal(
                            &transition_err.code,
                            &transition_err.message,
                            Some(&id),
                            vec![RemediationEntry {
                                command: format!("dont show {id}"),
                                description: "Inspect the current claim status".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    );
                }
                Ok(new_model_status) => {
                    let event = StoreEvent {
                        kind: StoreEventKind::Trusted,
                        note: Some(reason),
                        evidence: vec![],
                    };
                    let result = match project.store.append_status_change(
                        &id,
                        store_status_from_model(current),
                        store_status_from_model(new_model_status),
                        event,
                    ) {
                        Ok(r) => r,
                        Err(err) => handle_store_error(err, Some(&id)),
                    };
                    let updated = match project.store.claim_by_id(&id) {
                        Ok(Some(r)) => r,
                        Ok(None) => handle_store_error(
                            StoreError::Malformed(format!("claim {id} vanished after trust")),
                            Some(&id),
                        ),
                        Err(err) => handle_store_error(err, Some(&id)),
                    };
                    emit_claim_view(&updated, &result, &project.store);
                }
            }
        }

        Command::Forget { id } => {
            let project = open_project_or_exit();
            run_per_entity(id, |id| {
                if id.starts_with("term:") {
                    return emit_error_no_exit(
                        refusal(
                            "wrong-entity-kind",
                            "lock applies to claims only in this version",
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
                    Ok(None) => return emit_error_no_exit(
                        refusal(
                            "claim-not-found",
                            &format!("no claim with id {id}"),
                            Some(id),
                            vec![RemediationEntry {
                                command: "dont list".to_string(),
                                description: "List all claims to find the correct id".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                    Err(err) => return handle_store_error_code(err, Some(id)),
                };

                let current = model_status_from_store(record.status);
                match current {
                    Status::Locked => return emit_error_no_exit(
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
                    ),
                    Status::Verified => {}
                    _ => return emit_error_no_exit(
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
                    ),
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

                let result = match model_lock(current) {
                    Ok(new_model_status) => match project.store.append_status_change(
                        id,
                        store_status_from_model(current),
                        store_status_from_model(new_model_status),
                        StoreEvent {
                            kind: StoreEventKind::Locked,
                            note: None,
                            evidence: vec![],
                        },
                    ) {
                        Ok(r) => r,
                        Err(err) => return handle_store_error_code(err, Some(id)),
                    },
                    Err(err) => return emit_error_no_exit(
                        refusal(
                            &err.code,
                            &err.message,
                            Some(id),
                            vec![RemediationEntry {
                                command: format!("dont show {id}"),
                                description: "Inspect the current claim status".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                };

                let updated = match project.store.claim_by_id(id) {
                    Ok(Some(r)) => r,
                    Ok(None) => return handle_store_error_code(
                        StoreError::Malformed(format!("claim {id} vanished after lock")),
                        Some(id),
                    ),
                    Err(err) => return handle_store_error_code(err, Some(id)),
                };
                emit_claim_view(&updated, &result, &project.store);
                0
            });
        }

        Command::Reopen { id } => {
            let project = open_project_or_exit();

            if id.starts_with("term:") {
                let record = match project.store.term_by_id(&id) {
                    Ok(Some(r)) => r,
                    Ok(None) => emit_error_and_exit(
                        refusal(
                            "entity-not-found",
                            &format!("no entity with id {id}"),
                            Some(&id),
                            vec![RemediationEntry {
                                command: "dont list".to_string(),
                                description: "List all entities to find the correct id".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                    Err(err) => handle_store_error(err, Some(&id)),
                };
                let current = model_status_from_store(record.status);
                match model_reopen(current) {
                    Err(transition_err) => emit_error_and_exit(
                        refusal(
                            &transition_err.code,
                            &transition_err.message,
                            Some(&id),
                            vec![RemediationEntry {
                                command: format!("dont show {id}"),
                                description: "Inspect the current entity status".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                    Ok(new_model_status) => {
                        let event = StoreEvent {
                            kind: StoreEventKind::Reopened,
                            note: None,
                            evidence: vec![],
                        };
                        let result = match project.store.append_term_status_change(
                            &id,
                            store_status_from_model(current),
                            store_status_from_model(new_model_status),
                            event,
                        ) {
                            Ok(r) => r,
                            Err(err) => handle_store_error(err, Some(&id)),
                        };
                        let updated = match project.store.term_by_id(&id) {
                            Ok(Some(r)) => r,
                            Ok(None) => handle_store_error(
                                StoreError::Malformed(format!("term {id} vanished after reopen")),
                                Some(&id),
                            ),
                            Err(err) => handle_store_error(err, Some(&id)),
                        };
                        emit_term_view(&updated, &result, &project.store, vec![]);
                    }
                }
            } else {
                let record = match project.store.claim_by_id(&id) {
                    Ok(Some(r)) => r,
                    Ok(None) => emit_error_and_exit(
                        refusal(
                            "entity-not-found",
                            &format!("no entity with id {id}"),
                            Some(&id),
                            vec![RemediationEntry {
                                command: "dont list".to_string(),
                                description: "List all entities to find the correct id".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                    Err(err) => handle_store_error(err, Some(&id)),
                };
                let current = model_status_from_store(record.status);
                match model_reopen(current) {
                    Err(transition_err) => emit_error_and_exit(
                        refusal(
                            &transition_err.code,
                            &transition_err.message,
                            Some(&id),
                            vec![RemediationEntry {
                                command: format!("dont show {id}"),
                                description: "Inspect the current entity status".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                    Ok(new_model_status) => {
                        let event = StoreEvent {
                            kind: StoreEventKind::Reopened,
                            note: None,
                            evidence: vec![],
                        };
                        let result = match project.store.append_status_change(
                            &id,
                            store_status_from_model(current),
                            store_status_from_model(new_model_status),
                            event,
                        ) {
                            Ok(r) => r,
                            Err(err) => handle_store_error(err, Some(&id)),
                        };
                        let updated = match project.store.claim_by_id(&id) {
                            Ok(Some(r)) => r,
                            Ok(None) => handle_store_error(
                                StoreError::Malformed(format!("claim {id} vanished after reopen")),
                                Some(&id),
                            ),
                            Err(err) => handle_store_error(err, Some(&id)),
                        };
                        emit_claim_view(&updated, &result, &project.store);
                    }
                }
            }
        }

        Command::Ignore { id, reason } => {
            let reason = match reason {
                None => emit_error_and_exit(
                    refusal(
                        "reason-required",
                        "ignore requires --reason: state why this entity is being set aside",
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

            if id.starts_with("term:") {
                let record = match project.store.term_by_id(&id) {
                    Ok(Some(r)) => r,
                    Ok(None) => emit_error_and_exit(
                        refusal(
                            "entity-not-found",
                            &format!("no entity with id {id}"),
                            Some(&id),
                            vec![RemediationEntry {
                                command: "dont list".to_string(),
                                description: "List all entities to find the correct id".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                    Err(err) => handle_store_error(err, Some(&id)),
                };
                let current = model_status_from_store(record.status);
                match model_ignore(current) {
                    Err(transition_err) => emit_error_and_exit(
                        refusal(
                            &transition_err.code,
                            &transition_err.message,
                            Some(&id),
                            vec![RemediationEntry {
                                command: format!("dont show {id}"),
                                description: "Inspect the current entity status".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                    Ok(new_model_status) => {
                        let event = StoreEvent {
                            kind: StoreEventKind::Ignored,
                            note: Some(reason),
                            evidence: vec![],
                        };
                        let result = match project.store.append_term_status_change(
                            &id,
                            store_status_from_model(current),
                            store_status_from_model(new_model_status),
                            event,
                        ) {
                            Ok(r) => r,
                            Err(err) => handle_store_error(err, Some(&id)),
                        };
                        let updated = match project.store.term_by_id(&id) {
                            Ok(Some(r)) => r,
                            Ok(None) => handle_store_error(
                                StoreError::Malformed(format!("term {id} vanished after ignore")),
                                Some(&id),
                            ),
                            Err(err) => handle_store_error(err, Some(&id)),
                        };
                        emit_term_view(&updated, &result, &project.store, vec![]);
                    }
                }
            } else {
                let record = match project.store.claim_by_id(&id) {
                    Ok(Some(r)) => r,
                    Ok(None) => emit_error_and_exit(
                        refusal(
                            "entity-not-found",
                            &format!("no entity with id {id}"),
                            Some(&id),
                            vec![RemediationEntry {
                                command: "dont list".to_string(),
                                description: "List all entities to find the correct id".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                    Err(err) => handle_store_error(err, Some(&id)),
                };
                let current = model_status_from_store(record.status);
                match model_ignore(current) {
                    Err(transition_err) => emit_error_and_exit(
                        refusal(
                            &transition_err.code,
                            &transition_err.message,
                            Some(&id),
                            vec![RemediationEntry {
                                command: format!("dont show {id}"),
                                description: "Inspect the current entity status".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                    Ok(new_model_status) => {
                        let event = StoreEvent {
                            kind: StoreEventKind::Ignored,
                            note: Some(reason),
                            evidence: vec![],
                        };
                        let result = match project.store.append_status_change(
                            &id,
                            store_status_from_model(current),
                            store_status_from_model(new_model_status),
                            event,
                        ) {
                            Ok(r) => r,
                            Err(err) => handle_store_error(err, Some(&id)),
                        };
                        let updated = match project.store.claim_by_id(&id) {
                            Ok(Some(r)) => r,
                            Ok(None) => handle_store_error(
                                StoreError::Malformed(format!("claim {id} vanished after ignore")),
                                Some(&id),
                            ),
                            Err(err) => handle_store_error(err, Some(&id)),
                        };
                        emit_claim_view(&updated, &result, &project.store);
                    }
                }
            }
        }

        Command::Flag { id, evidence, file, lines, anchor, excerpt } => {
            if evidence.is_empty() && file.is_none() {
                emit_error_and_exit(
                    refusal(
                        "no-evidence",
                        "flag requires at least one --evidence URI or --file locator",
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

            let project = open_project_or_exit();
            let project_root = project.dont_dir.parent().unwrap_or(&project.dont_dir).to_path_buf();

            // Build the full evidence list, appending structured locator if --file was given.
            let mut all_evidence: Vec<Value> =
                evidence.into_iter().map(Value::String).collect();
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
            // Terms don't have depends_on fields so no dependency gate is needed here.
            // If terms gain dependencies in the future, add dependency_gate_unmet_clauses.
            if id.starts_with("term:") {
                let record = match project.store.term_by_id(&id) {
                    Ok(Some(r)) => r,
                    Ok(None) => emit_error_and_exit(
                        refusal(
                            "term-not-found",
                            &format!("no term with id {id}"),
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
                };

                let current = model_status_from_store(record.status);
                let event = StoreEvent {
                    kind: StoreEventKind::Flagged,
                    note: None,
                    evidence: all_evidence.clone(),
                };

                let result = match model_flag(current) {
                    Ok(new_model_status) => match project.store.append_term_status_change(
                        &id,
                        store_status_from_model(current),
                        store_status_from_model(new_model_status),
                        event,
                    ) {
                        Ok(r) => r,
                        Err(err) => handle_store_error(err, Some(&id)),
                    },
                    Err(transition_err) => {
                        emit_error_and_exit(
                            refusal(
                                &transition_err.code,
                                &transition_err.message,
                                Some(&id),
                                vec![RemediationEntry {
                                    command: format!("dont show {id}"),
                                    description: "Inspect the current term status".to_string(),
                                }],
                            ),
                            vec![],
                            1,
                        );
                    }
                };

                let updated = match project.store.term_by_id(&id) {
                    Ok(Some(r)) => r,
                    Ok(None) => handle_store_error(
                        StoreError::Malformed(format!("term {id} vanished after flag")),
                        Some(&id),
                    ),
                    Err(err) => handle_store_error(err, Some(&id)),
                };
                emit_term_view(&updated, &result, &project.store, vec![]);
                return;
            }

            let record = match project.store.claim_by_id(&id) {
                Ok(Some(r)) => r,
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
            };

            let current = model_status_from_store(record.status);
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
                        description: "Inspect the blocking dependency assessments"
                            .to_string(),
                    }],
                )
                .expect("dependency gate refusal must include remediation");
                emit_error_and_exit(err_result, vec![], 1);
            }
            let event = StoreEvent {
                kind: StoreEventKind::Flagged,
                note: None,
                evidence: all_evidence,
            };

            let result = match model_flag(current) {
                Ok(new_model_status) => {
                    match project.store.append_status_change(
                        &id,
                        store_status_from_model(current),
                        store_status_from_model(new_model_status),
                        event,
                    ) {
                        Ok(r) => r,
                        Err(err) => handle_store_error(err, Some(&id)),
                    }
                }
                Err(_) if current == Status::Verified => {
                    // already-verified flag appends evidence without status change
                    match project.store.append_evidence_event(&id, event) {
                        Ok(r) => r,
                        Err(err) => handle_store_error(err, Some(&id)),
                    }
                }
                Err(transition_err) => {
                    emit_error_and_exit(
                        refusal(
                            &transition_err.code,
                            &transition_err.message,
                            Some(&id),
                            vec![RemediationEntry {
                                command: format!("dont show {id}"),
                                description: "Inspect the current claim status".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    );
                }
            };

            let updated = match project.store.claim_by_id(&id) {
                Ok(Some(r)) => r,
                Ok(None) => handle_store_error(
                    StoreError::Malformed(format!("claim {id} vanished after flag")),
                    Some(&id),
                ),
                Err(err) => handle_store_error(err, Some(&id)),
            };
            emit_claim_view(&updated, &result, &project.store);
        }

        Command::Undoubt { id } => {
            let project = open_project_or_exit();

            if id.starts_with("term:") {
                let record = match project.store.term_by_id(&id) {
                    Ok(Some(r)) => r,
                    Ok(None) => emit_error_and_exit(
                        refusal(
                            "entity-not-found",
                            &format!("no entity with id {id}"),
                            Some(&id),
                            vec![RemediationEntry {
                                command: "dont list".to_string(),
                                description: "List all entities to find the correct id".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                    Err(err) => handle_store_error(err, Some(&id)),
                };
                let current = model_status_from_store(record.status);
                match model_undoubt(current) {
                    Err(transition_err) => emit_error_and_exit(
                        refusal(
                            &transition_err.code,
                            &transition_err.message,
                            Some(&id),
                            vec![RemediationEntry {
                                command: format!("dont show {id}"),
                                description: "Inspect the current entity status".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                    Ok(new_model_status) => {
                        let event = StoreEvent {
                            kind: StoreEventKind::Undoubted,
                            note: None,
                            evidence: vec![],
                        };
                        let result = match project.store.append_term_status_change(
                            &id,
                            store_status_from_model(current),
                            store_status_from_model(new_model_status),
                            event,
                        ) {
                            Ok(r) => r,
                            Err(err) => handle_store_error(err, Some(&id)),
                        };
                        let updated = match project.store.term_by_id(&id) {
                            Ok(Some(r)) => r,
                            Ok(None) => handle_store_error(
                                StoreError::Malformed(format!("term {id} vanished after undoubt")),
                                Some(&id),
                            ),
                            Err(err) => handle_store_error(err, Some(&id)),
                        };
                        emit_term_view(&updated, &result, &project.store, vec![]);
                    }
                }
            } else {
                let record = match project.store.claim_by_id(&id) {
                    Ok(Some(r)) => r,
                    Ok(None) => emit_error_and_exit(
                        refusal(
                            "entity-not-found",
                            &format!("no entity with id {id}"),
                            Some(&id),
                            vec![RemediationEntry {
                                command: "dont list".to_string(),
                                description: "List all entities to find the correct id".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                    Err(err) => handle_store_error(err, Some(&id)),
                };
                let current = model_status_from_store(record.status);
                match model_undoubt(current) {
                    Err(transition_err) => emit_error_and_exit(
                        refusal(
                            &transition_err.code,
                            &transition_err.message,
                            Some(&id),
                            vec![RemediationEntry {
                                command: format!("dont show {id}"),
                                description: "Inspect the current entity status".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                    Ok(new_model_status) => {
                        let event = StoreEvent {
                            kind: StoreEventKind::Undoubted,
                            note: None,
                            evidence: vec![],
                        };
                        let result = match project.store.append_status_change(
                            &id,
                            store_status_from_model(current),
                            store_status_from_model(new_model_status),
                            event,
                        ) {
                            Ok(r) => r,
                            Err(err) => handle_store_error(err, Some(&id)),
                        };
                        let updated = match project.store.claim_by_id(&id) {
                            Ok(Some(r)) => r,
                            Ok(None) => handle_store_error(
                                StoreError::Malformed(format!("claim {id} vanished after undoubt")),
                                Some(&id),
                            ),
                            Err(err) => handle_store_error(err, Some(&id)),
                        };
                        emit_claim_view(&updated, &result, &project.store);
                    }
                }
            }
        }

        Command::Show { id } => {
            let project = open_project_or_exit();
            run_per_entity(id, |id| {
                match project.store.resolve_entity(id) {
                    Ok(Some(EntityResolution::Claim(record))) => {
                        let payload = build_claim_view(&record, &project.store);
                        let env = Envelope::success(
                            "claim",
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
                            "term",
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
                }
            });
        }

        Command::Why { id } => {
            let project = open_project_or_exit();
            run_per_entity(id, |id| {
                match project.store.resolve_entity(id) {
                    Ok(Some(EntityResolution::Claim(record))) => {
                        let payload = build_claim_why_view(&record, &project.store);
                        let env = Envelope::success(
                            "why",
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
                            "why",
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
                }
            });
        }

        Command::VerifyEvidence {
            id,
            timeout_seconds,
        } => {
            let project = open_project_or_exit();
            let config = project.load_config();
            let effective_timeout = timeout_seconds.or(config.verify_evidence.default_timeout_s);

            let (entity_kind, status, evidence) = if id.starts_with("term:") {
                match project.store.term_by_id(&id) {
                    Ok(Some(record)) => (
                        "term",
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
                        "claim",
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
                let remediation = if entity_kind == "claim" {
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
                .map(|result| serde_json::to_value(result).expect("evidence check result serializes"))
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
                "evidence_check",
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
                    StoreStatus::Unverified => unverified += 1,
                    StoreStatus::Doubted => {
                        doubted += 1;
                        blocking.push(json!({
                            "id": claim.id,
                            "statement": claim.statement,
                            "status": "doubted",
                        }));
                    }
                    StoreStatus::Verified => verified += 1,
                    StoreStatus::Ignored => ignored += 1,
                    StoreStatus::Locked => locked += 1,
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
                    StoreStatus::Unverified => unverified += 1,
                    StoreStatus::Doubted => {
                        doubted += 1;
                        blocking.push(json!({
                            "id": term.id,
                            "curie": term.curie,
                            "status": "doubted",
                        }));
                    }
                    StoreStatus::Verified => verified += 1,
                    StoreStatus::Ignored => ignored += 1,
                    StoreStatus::Locked => locked += 1,
                }
                let projected =
                    project_evidence(collect_term_evidence(term), &project_root);
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
                "mode": project.mode(),
                "status_counts": {
                    "unverified": unverified,
                    "doubted": doubted,
                    "verified": verified,
                    "locked": locked,
                    "ignored": ignored,
                },
                "assessment_counts": {
                    "stale": ac_stale,
                    "compromised_support": ac_compromised,
                    "dangling_dependency": ac_dangling,
                    "unresolved_term": ac_unresolved,
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
            let env = Envelope::success("prime", payload, vec![], vec![]);
            emit_json(&env);
            if !blocking.is_empty() {
                std::process::exit(1);
            }
        }

        Command::List { status, kind } => {
            let project = open_project_or_exit();
            let status_filter = match status {
                Some(raw) => match parse_claim_status_filter(&raw) {
                    Some(status) => Some(status),
                    None => emit_error_and_exit(
                        refusal(
                            "invalid-status",
                            &format!(
                                "unsupported claim status '{raw}'; expected one of: unverified, verified, doubted, ignored"
                            ),
                            None,
                            vec![RemediationEntry {
                                command: "dont list --status unverified".to_string(),
                                description:
                                    "Use one of: unverified, verified, doubted, ignored"
                                        .to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                },
                None => None,
            };
            let default_kind = kind.is_none();
            let list_kind = match kind {
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
                    // Sort by created_at descending; use id (ULID) as tiebreaker within same second
                    claims.sort_by(|a, b| {
                        b.created_at
                            .cmp(&a.created_at)
                            .then_with(|| b.id.cmp(&a.id))
                    });
                    let views: Vec<Value> = claims.iter().map(|c| build_claim_view(c, &project.store)).collect();
                    let hints = match project.store.list_terms() {
                        Ok(terms) if default_kind && !terms.is_empty() => vec![HintEntry {
                            command: "dont list --kind terms".to_string(),
                            description: "List defined term entities as well".to_string(),
                        }],
                        Ok(_) => vec![],
                        Err(err) => handle_store_error(err, None),
                    };
                    let env = Envelope::success("claims", views, vec![], hints);
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
                    let views: Vec<Value> = terms.iter().map(|term| build_term_view(term, &project.store)).collect();
                    let env = Envelope::success("terms", views, vec![], vec![]);
                    emit_json(&env);
                }
            }
        }

        Command::Trace { id } => {
            let project = open_project_or_exit();
            if id.starts_with("term:") {
                match project.store.term_by_id(&id) {
                    Ok(Some(_)) => {
                        let payload = json!({
                            "entity_id": id,
                            "blockers": [],
                            "as_of": chrono::Utc::now().to_rfc3339(),
                        });
                        let env = Envelope::success("trace", payload, vec![], vec![]);
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
                        let env = Envelope::success("trace", payload, vec![], hints);
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
                emit_json(&Envelope::success("dont-completions", payload, vec![], vec![]));
            } else {
                clap_complete::generate(shell, &mut cmd, "dont", &mut std::io::stdout());
            }
        }

        Command::Ground {
            statement,
            evidence,
            file,
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
                            description: "Provide the statement directly as an argument".to_string(),
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
                        "ground requires a non-empty claim statement",
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

            let evidence: Vec<String> = evidence
                .into_iter()
                .filter(|e| !e.trim().is_empty())
                .collect();

            if evidence.is_empty() && file.is_none() {
                emit_error_and_exit(
                    refusal(
                        "no-evidence",
                        "ground requires at least one --evidence URI or --file locator",
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

            let project = open_project_or_exit();
            let project_root = project.dont_dir.parent().unwrap_or(&project.dont_dir).to_path_buf();

            let mut all_evidence: Vec<Value> =
                evidence.into_iter().map(Value::String).collect();
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

            // Write claim then immediately verify — both or neither.
            let conclude_result = match project.store.append_claim(&statement, &[]) {
                Ok(r) => r,
                Err(err) => handle_store_error(err, None),
            };
            let claim_id = conclude_result.id.clone();

            let flag_event = StoreEvent {
                kind: StoreEventKind::Flagged,
                note: None,
                evidence: all_evidence,
            };
            let flag_result = match project.store.append_status_change(
                &claim_id,
                StoreStatus::Unverified,
                StoreStatus::Verified,
                flag_event,
            ) {
                Ok(r) => r,
                Err(err) => handle_store_error(err, Some(&claim_id)),
            };

            let updated = match project.store.claim_by_id(&claim_id) {
                Ok(Some(r)) => r,
                Ok(None) => handle_store_error(
                    StoreError::Malformed(format!("claim {claim_id} vanished after ground")),
                    Some(&claim_id),
                ),
                Err(err) => handle_store_error(err, Some(&claim_id)),
            };
            emit_claim_view(&updated, &flag_result, &project.store);
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
                                    command: format!("dont atom dismiss {id} {idx} --evidence <uri>"),
                                    description: "Attach evidence for the atom verification".to_string(),
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
                                        description: "Inspect the claim to see available atom indices".to_string(),
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
                            StoreError::Malformed(format!("claim {id} vanished after atom dismiss")),
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
                            StoreError::Malformed(format!("claim {id} vanished after hypothesis add")),
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
                    let result = match project.store.assess_hypothesis(&id, idx, &supporting, &refuting) {
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
                                    "hypothesis-not-found",
                                    &format!("hypothesis index {idx} does not exist on claim {id}"),
                                    Some(&id),
                                    vec![RemediationEntry {
                                        command: format!("dont show {id}"),
                                        description: "Inspect the claim to see available hypothesis indices".to_string(),
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
                            StoreError::Malformed(format!("claim {id} vanished after hypothesis assess")),
                            Some(&id),
                        ),
                        Err(err) => handle_store_error(err, Some(&id)),
                    };
                    emit_claim_view(&updated, &result, &project.store);
                }
            }
        }

        Command::Import { adapter, .. } => {
            let project = open_project_or_exit();
            let config = project.load_config();
            let adapter_cfg = config.import.adapters.get(&adapter).cloned().unwrap_or_default();
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
            emit_error_and_exit(
                refusal(
                    "not-implemented",
                    &format!("import adapter '{adapter}' is not yet implemented"),
                    None,
                    vec![],
                ),
                vec![],
                1,
            );
        }

        Command::Rules { action } => {
            let project = open_project_or_exit();
            let rules_dir = project.dont_dir.join("rules");
            let config = project.load_config();
            let engine = dont::rules::RuleEngine::new(
                rules_dir.clone(),
                config.rules,
                project.mode() == "strict",
            );

            match action {
                RulesAction::List => {
                    let mut rules: Vec<RuleInfo> = SHIPPED_RULES
                        .iter()
                        .map(|name| RuleInfo {
                            name: name.to_string(),
                            severity: severity_label(engine.severity(name)),
                            source: "shipped",
                        })
                        .collect();

                    if let Ok(entries) = std::fs::read_dir(&rules_dir) {
                        let mut custom: Vec<RuleInfo> = entries
                            .filter_map(|e| e.ok())
                            .filter(|e| {
                                e.path().extension().and_then(|x| x.to_str()) == Some("dl")
                            })
                            .filter_map(|e| {
                                let stem = e.path().file_stem()?.to_str()?.to_string();
                                if SHIPPED_RULES.contains(&stem.as_str()) {
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

                    emit_json(&Envelope::success("rule_list", rules, vec![], vec![]));
                }

                RulesAction::Show { name } => {
                    if SHIPPED_RULES.contains(&name.as_str()) {
                        let detail = RuleDetail {
                            name: name.clone(),
                            severity: severity_label(engine.severity(&name)),
                            source: "shipped",
                            datalog: None,
                        };
                        emit_json(&Envelope::success("rule", detail, vec![], vec![]));
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
                                emit_json(&Envelope::success("rule", detail, vec![], vec![]));
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

                    if SHIPPED_RULES.contains(&rule_name) {
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
                                    command: format!(
                                        "dont rules add {} --force",
                                        file.display()
                                    ),
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

                    emit_json(&Envelope::success(
                        "empty",
                        serde_json::Value::Null,
                        vec![],
                        vec![HintEntry {
                            command: "dont rules list".to_string(),
                            description: format!("Rule {rule_name:?} is now active"),
                        }],
                    ));
                }

                RulesAction::Test { name } => {
                    let matches = match engine.evaluate_shipped(&project.store, &name) {
                        Some(Ok(m)) => m,
                        Some(Err(e)) => emit_error_and_exit(
                            refusal(
                                "rule-eval-error",
                                &e.to_string(),
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
                                    &e.to_string(),
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
                    emit_json(&Envelope::success("rule_result", result, vec![], vec![]));
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
                project.mode() == "strict",
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
                    emit_json(&Envelope::success("dont-explain", payload, vec![], vec![]));
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
    }
}
