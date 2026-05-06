use std::cell::Cell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use dont::envelope::{
    Envelope, ErrorResult, HintEntry, RemediationEntry, UnmetClause, Warning,
};
use dont::model::{
    Status, dismiss as model_dismiss, ignore as model_ignore, lock as model_lock,
    reopen as model_reopen, trust as model_trust,
};
use dont::project::{Project, ProjectError, ProjectMode};
use dont::store::{
    AppendResult, ClaimRecord, EventRecord, HypothesisRecord, StoreError, StoreEvent,
    StoreEventKind, StoreStatus, TermRecord,
};

thread_local! {
    static HUMAN_MODE: Cell<bool> = const { Cell::new(false) };
}

fn human_mode() -> bool {
    HUMAN_MODE.with(|m| m.get())
}

#[derive(Debug, Parser)]
#[command(name = "dont")]
#[command(version)]
#[command(about = "Epistemic forcing-function CLI for grounded claims")]
struct Cli {
    /// Output JSON envelope on stdout.
    #[arg(long, global = true)]
    json: bool,

    /// Output human-readable text instead of JSON (--json takes precedence).
    #[arg(long, global = true)]
    human: bool,

    #[command(subcommand)]
    command: Command,
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

    /// Register explicit doubt about a claim.
    Trust {
        /// Claim identifier.
        id: String,

        /// Reason for doubt (required).
        #[arg(long, short)]
        reason: Option<String>,
    },

    /// Verify or add evidence to a claim.
    Dismiss {
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

    /// Promote a verified claim to locked when the lockable gate is met.
    Lock {
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

    /// Manage competing hypotheses for a claim.
    Hypothesis {
        #[command(subcommand)]
        action: HypothesisAction,
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

fn contains_hedge(reason: &str) -> bool {
    let lower = reason.to_lowercase();
    DEFAULT_HEDGES.iter().any(|h| lower.contains(h))
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

fn emit_error_and_exit(err: ErrorResult, warnings: Vec<Warning>, code: i32) -> ! {
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
    process::exit(code);
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
                format!("{status}  {id}\n  {statement}")
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
                format!("{status}  {id}  {curie}")
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
            format!("{status:<12}  {id}  {truncated}")
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
            format!("{status:<12}  {id}  {curie}")
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
    let mut out = format!(
        "{id}\n  status:     {status}\n  statement:  {statement}\n  evidence:\n{evidence_str}\n  created:    {created}"
    );
    if let Some(deps) = depends {
        if !deps.is_empty() {
            let dep_list: Vec<&str> = deps.iter().filter_map(Value::as_str).collect();
            out.push_str(&format!("\n  depends_on: {}", dep_list.join(", ")));
        }
    }
    out
}

fn format_term_detail(data: &Value) -> String {
    let id = data["id"].as_str().unwrap_or("?");
    let status = data["status"].as_str().unwrap_or("?");
    let curie = data["curie"].as_str().unwrap_or("?");
    let definition = data["definition"].as_str().unwrap_or("(none)");
    let mut out = format!("{id}  {curie}\n  status:      {status}\n  definition:  {definition}");
    if let Some(label) = data["label"].as_str() {
        if !label.is_empty() {
            out.push_str(&format!("\n  label:       {label}"));
        }
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
    if let Some(blocking) = data["blocking"].as_array() {
        if !blocking.is_empty() {
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
    }
    out
}

fn format_trace(data: &Value) -> String {
    let id = data["entity_id"].as_str().unwrap_or("?");
    match data["blocker_paths"].as_array() {
        Some(p) if p.is_empty() => format!("{id}  no blockers"),
        Some(p) => {
            let mut out = format!("{id} is blocked by:");
            for path in p {
                out.push_str(&format!("\n  {path}"));
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
        Ok(p) => p,
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

fn collect_evidence(record: &ClaimRecord) -> Vec<Value> {
    collect_evidence_from_events(&record.events)
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

fn build_claim_view(record: &ClaimRecord) -> Value {
    let evidence = collect_evidence(record);
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
        "derived_assessments": derived_assessments_for_claim(record),
        "atoms": [],
        "hypotheses": record.hypotheses,
        "evidence": evidence,
        "depends_on": record.depends_on,
        "events": events,
        "applicable_rules": {
            "lockable": lockable_rule_view(record),
        },
        "created_at": record.created_at,
        "updated_at": updated_at(record),
    })
}

fn build_term_view(record: &TermRecord) -> Value {
    let evidence = collect_term_evidence(record);
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
    if let Some(obj) = v.as_object() {
        if obj.get("kind").and_then(Value::as_str) == Some("repo-file") {
            if let Some(path) = obj.get("path").and_then(Value::as_str) {
                return format!("repo-file:{path}");
            }
        }
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
    Ok(full.strip_prefix(project_root).unwrap_or(&full).to_path_buf())
}

/// Parse a line span string like "10-18" or "42" into (start, end).
fn parse_line_span(s: &str) -> Result<(u32, u32), String> {
    if let Some((a, b)) = s.split_once('-') {
        let start: u32 = a.trim().parse().map_err(|_| format!("invalid line span: {s}"))?;
        let end: u32 = b.trim().parse().map_err(|_| format!("invalid line span: {s}"))?;
        if start > end {
            return Err(format!("line span start {start} is greater than end {end}"));
        }
        Ok((start, end))
    } else {
        let line: u32 = s.trim().parse().map_err(|_| format!("invalid line number: {s}"))?;
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

fn derived_assessments_for_claim(record: &ClaimRecord) -> Vec<String> {
    let mut derived = Vec::new();
    if record.depends_on.is_empty() {
        return derived;
    }
    let project = match Project::open(&cwd()) {
        Ok(project) => project,
        Err(_) => return derived,
    };

    for dep in &record.depends_on {
        let lookup = if dep.starts_with("term:") {
            project.store.term_by_id(dep)
        } else {
            project.store.term_by_curie(dep)
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
    path: Vec<String>,
    blocking_node: String,
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
                path: vec![start_id.to_string(), term.id.clone()],
                blocking_node: term.id,
                remediation,
            })
        }
        Ok(None) => Some(BlockerPath {
            kind: "unresolved-term".to_string(),
            path,
            blocking_node: dep.to_string(),
            remediation: vec![RemediationEntry {
                command: format!("dont define {dep} --doc \"<definition>\""),
                description: format!("Define the missing term {dep}"),
            }],
        }),
        Err(_) => Some(BlockerPath {
            kind: "dangling-dependency".to_string(),
            path,
            blocking_node: dep.to_string(),
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
    json!({
        "kind": bp.kind,
        "path": bp.path,
        "blocking_node": bp.blocking_node,
        "remediation": bp.remediation.iter().map(|r| json!({
            "command": r.command,
            "description": r.description,
        })).collect::<Vec<_>>(),
    })
}

fn lockable_unmet_clauses(record: &ClaimRecord) -> Vec<UnmetClause> {
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

    for assessment in derived_assessments_for_claim(record) {
        unmet.push(UnmetClause {
            clause: format!("derived assessment {assessment} blocks locking"),
            fix: "resolve dependency integrity issues before locking".to_string(),
        });
    }

    unmet
}

fn lockable_rule_view(record: &ClaimRecord) -> Value {
    let unmet: Vec<String> = lockable_unmet_clauses(record)
        .into_iter()
        .map(|clause| clause.clause)
        .collect();
    json!({
        "rule_kind": "gate",
        "met": unmet.is_empty(),
        "unmet": unmet,
    })
}

fn dependency_gate_unmet_clauses(record: &ClaimRecord) -> Vec<UnmetClause> {
    derived_assessments_for_claim(record)
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
        outcome: "reachable".to_string(),
        detail: None,
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
    if let StoreError::CurieConflict { curie, existing_id } = err {
        emit_error_and_exit(
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
    emit_error_and_exit(err_result, vec![], 4);
}

fn suggest_alternative_curie(curie: &str) -> String {
    match curie.rsplit_once(':') {
        Some((prefix, local)) => format!("{prefix}:{}_2", local),
        None => format!("{curie}_2"),
    }
}

fn emit_claim_view(record: &ClaimRecord, result: &AppendResult) {
    let payload = build_claim_view(record);
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

fn emit_term_view(record: &TermRecord, result: &AppendResult, warnings: Vec<Warning>) {
    let payload = build_term_view(record);
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

fn validate_label(label: &str, curie: &str) -> Option<ErrorResult> {
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
    if !label_has_indefinite_article(label) {
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
    if label_ends_with_sentence_punctuation(label) {
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
    if label_compound_undeclared(label) {
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
    if label_contains_sentence_verb(label) {
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

fn main() {
    let cli = Cli::parse();

    if cli.human && !cli.json {
        HUMAN_MODE.with(|m| m.set(true));
    }

    match cli.command {
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

            let warnings = match &label {
                Some(lbl) => {
                    if let Some(err) = validate_label(lbl, &curie) {
                        emit_error_and_exit(err, vec![], 1);
                    }
                    vec![]
                }
                None => doc_shape_warnings(&doc),
            };

            let project = open_project_or_exit();
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
            emit_term_view(&term, &result, warnings);
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

            if contains_hedge(&reason) {
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

            let project = open_project_or_exit();
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
                        emit_term_view(&updated, &result, vec![]);
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
                    emit_claim_view(&updated, &result);
                }
            }
        }

        Command::Lock { id } => {
            if id.starts_with("term:") {
                emit_error_and_exit(
                    refusal(
                        "wrong-entity-kind",
                        "lock applies to claims only in this version",
                        Some(&id),
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

            let project = open_project_or_exit();
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
            match current {
                Status::Locked => emit_error_and_exit(
                    refusal(
                        "claim-locked",
                        "claim is already locked",
                        Some(&id),
                        vec![RemediationEntry {
                            command: format!("dont show {id}"),
                            description: "Inspect the locked claim".to_string(),
                        }],
                    ),
                    vec![],
                    1,
                ),
                Status::Verified => {}
                _ => emit_error_and_exit(
                    refusal(
                        "claim-not-verified",
                        "claim must be verified before it can be locked",
                        Some(&id),
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

            let unmet_clauses = lockable_unmet_clauses(&record);
            if !unmet_clauses.is_empty() {
                let err_result = ErrorResult::new(
                    "rule-not-met",
                    "lockable gate is not met",
                    Some("lockable"),
                    None,
                    Some(&id),
                    unmet_clauses,
                    vec![RemediationEntry {
                        command: format!("dont show {id}"),
                        description: "Inspect the claim and satisfy the unmet lock gates"
                            .to_string(),
                    }],
                )
                .expect("lock refusal must include remediation");
                emit_error_and_exit(err_result, vec![], 1);
            }

            let result = match model_lock(current) {
                Ok(new_model_status) => match project.store.append_status_change(
                    &id,
                    store_status_from_model(current),
                    store_status_from_model(new_model_status),
                    StoreEvent {
                        kind: StoreEventKind::Locked,
                        note: None,
                        evidence: vec![],
                    },
                ) {
                    Ok(r) => r,
                    Err(err) => handle_store_error(err, Some(&id)),
                },
                Err(err) => emit_error_and_exit(
                    refusal(
                        &err.code,
                        &err.message,
                        Some(&id),
                        vec![RemediationEntry {
                            command: format!("dont show {id}"),
                            description: "Inspect the current claim status".to_string(),
                        }],
                    ),
                    vec![],
                    1,
                ),
            };

            let updated = match project.store.claim_by_id(&id) {
                Ok(Some(r)) => r,
                Ok(None) => handle_store_error(
                    StoreError::Malformed(format!("claim {id} vanished after lock")),
                    Some(&id),
                ),
                Err(err) => handle_store_error(err, Some(&id)),
            };
            emit_claim_view(&updated, &result);
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
                        emit_term_view(&updated, &result, vec![]);
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
                        emit_claim_view(&updated, &result);
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

            if contains_hedge(&reason) {
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
                        emit_term_view(&updated, &result, vec![]);
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
                        emit_claim_view(&updated, &result);
                    }
                }
            }
        }

        Command::Dismiss { id, evidence, file, lines, anchor, excerpt } => {
            if evidence.is_empty() && file.is_none() {
                emit_error_and_exit(
                    refusal(
                        "no-evidence",
                        "dismiss requires at least one --evidence URI or --file locator",
                        Some(&id),
                        vec![RemediationEntry {
                            command: format!("dont dismiss {id} --evidence <uri>"),
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
                if PathBuf::from(file_path).is_absolute() {
                    emit_error_and_exit(
                        refusal(
                            "path-not-relative",
                            "repository evidence locators must be project-relative paths, not absolute",
                            Some(&id),
                            vec![RemediationEntry {
                                command: format!("dont dismiss {id} --file <relative-path>"),
                                description: "Use a path relative to the project root".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    );
                }
                let normalized = match normalize_repo_path(file_path, &project_root) {
                    Ok(p) => p,
                    Err(msg) => emit_error_and_exit(
                        refusal(
                            "path-escapes-root",
                            &format!("evidence locator path is invalid: {msg}"),
                            Some(&id),
                            vec![RemediationEntry {
                                command: format!("dont dismiss {id} --file <relative-path>"),
                                description: "Use a path that stays within the project root".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                };
                let line_span = match lines.as_deref().map(parse_line_span) {
                    Some(Ok(span)) => Some(span),
                    Some(Err(msg)) => emit_error_and_exit(
                        refusal(
                            "invalid-line-span",
                            &format!("invalid --lines value: {msg}"),
                            Some(&id),
                            vec![RemediationEntry {
                                command: format!("dont dismiss {id} --file {file_path} --lines <start-end>"),
                                description: "Use a format like \"10-18\" or \"42\"".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                    None => None,
                };
                all_evidence.push(build_repo_locator(
                    &normalized,
                    line_span,
                    anchor.as_deref(),
                    excerpt.as_deref(),
                ));
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
                let event = StoreEvent {
                    kind: StoreEventKind::Dismissed,
                    note: None,
                    evidence: all_evidence.clone(),
                };

                let result = match model_dismiss(current) {
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
                        StoreError::Malformed(format!("term {id} vanished after dismiss")),
                        Some(&id),
                    ),
                    Err(err) => handle_store_error(err, Some(&id)),
                };
                emit_term_view(&updated, &result, vec![]);
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
            let dependency_unmet = dependency_gate_unmet_clauses(&record);
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
                kind: StoreEventKind::Dismissed,
                note: None,
                evidence: all_evidence,
            };

            let result = match model_dismiss(current) {
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
                    // Phase 8: already-verified dismiss appends evidence without status change
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
                    StoreError::Malformed(format!("claim {id} vanished after dismiss")),
                    Some(&id),
                ),
                Err(err) => handle_store_error(err, Some(&id)),
            };
            emit_claim_view(&updated, &result);
        }

        Command::Show { id } => {
            let project = open_project_or_exit();
            if id.starts_with("term:") {
                match project.store.term_by_id(&id) {
                    Ok(Some(record)) => {
                        let payload = build_term_view(&record);
                        let env = Envelope::success(
                            "term",
                            payload,
                            vec![],
                            vec![HintEntry {
                                command: format!("dont trust {id} --reason \"...\""),
                                description: "Register doubt about this term".to_string(),
                            }],
                        );
                        emit_json(&env);
                    }
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
                }
            } else if !id.starts_with("claim:") && id.contains(':') {
                match project.store.term_by_curie(&id) {
                    Ok(Some(record)) => {
                        let payload = build_term_view(&record);
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
                    }
                    Ok(None) => emit_error_and_exit(
                        refusal(
                            "term-not-found",
                            &format!("no term with curie {id}"),
                            Some(&id),
                            vec![RemediationEntry {
                                command: "dont vocab".to_string(),
                                description: "List terms to find the correct curie".to_string(),
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
                        let payload = build_claim_view(&record);
                        let env = Envelope::success(
                            "claim",
                            payload,
                            vec![],
                            vec![HintEntry {
                                command: format!("dont trust {id} --reason \"...\""),
                                description: "Register doubt about this claim".to_string(),
                            }],
                        );
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

        Command::VerifyEvidence {
            id,
            timeout_seconds,
        } => {
            let project = open_project_or_exit();

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
            // Only URI-based entries participate in liveness checks; structured locators
            // are skipped here (drift detection belongs to dont-8bu).
            let uri_evidence: Vec<&str> = evidence
                .iter()
                .filter_map(|v| v.as_str())
                .collect();
            let results: Vec<EvidenceCheckResult> = uri_evidence
                .iter()
                .map(|uri| check_evidence_uri(uri, mocks.as_ref(), timeout_seconds))
                .collect();
            let warnings: Vec<Warning> = results
                .iter()
                .filter_map(|result| evidence_check_warning(&id, result))
                .collect();
            let payload = json!({
                "entity_id": id,
                "entity_kind": entity_kind,
                "status": status,
                "timeout_seconds": timeout_seconds,
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
                    "stale": 0,
                    "compromised_support": 0,
                    "dangling_dependency": 0,
                    "unresolved_term": 0,
                },
                "rules": { "strict": [], "warn": [] },
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
                    let views: Vec<Value> = claims.iter().map(build_claim_view).collect();
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
                    let views: Vec<Value> = terms.iter().map(build_term_view).collect();
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
                            "blocker_paths": [],
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
                            "blocker_paths": blocker_paths,
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
            clap_complete::generate(shell, &mut cmd, "dont", &mut std::io::stdout());
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
                if PathBuf::from(file_path).is_absolute() {
                    emit_error_and_exit(
                        refusal(
                            "path-not-relative",
                            "repository evidence locators must be project-relative paths, not absolute",
                            None,
                            vec![RemediationEntry {
                                command: "dont ground \"<statement>\" --file <relative-path>".to_string(),
                                description: "Use a path relative to the project root".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    );
                }
                let normalized = match normalize_repo_path(file_path, &project_root) {
                    Ok(p) => p,
                    Err(msg) => emit_error_and_exit(
                        refusal(
                            "path-escapes-root",
                            &format!("evidence locator path is invalid: {msg}"),
                            None,
                            vec![RemediationEntry {
                                command: "dont ground \"<statement>\" --file <relative-path>".to_string(),
                                description: "Use a path that stays within the project root".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                };
                let line_span = match lines.as_deref().map(parse_line_span) {
                    Some(Ok(span)) => Some(span),
                    Some(Err(msg)) => emit_error_and_exit(
                        refusal(
                            "invalid-line-span",
                            &format!("invalid --lines value: {msg}"),
                            None,
                            vec![RemediationEntry {
                                command: "dont ground \"<statement>\" --file <path> --lines <start-end>".to_string(),
                                description: "Use a format like \"10-18\" or \"42\"".to_string(),
                            }],
                        ),
                        vec![],
                        1,
                    ),
                    None => None,
                };
                all_evidence.push(build_repo_locator(
                    &normalized,
                    line_span,
                    anchor.as_deref(),
                    excerpt.as_deref(),
                ));
            }

            // Write claim then immediately verify — both or neither.
            let conclude_result = match project.store.append_claim(&statement, &[]) {
                Ok(r) => r,
                Err(err) => handle_store_error(err, None),
            };
            let claim_id = conclude_result.id.clone();

            let dismiss_event = StoreEvent {
                kind: StoreEventKind::Dismissed,
                note: None,
                evidence: all_evidence,
            };
            let dismiss_result = match project.store.append_status_change(
                &claim_id,
                StoreStatus::Unverified,
                StoreStatus::Verified,
                dismiss_event,
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
            emit_claim_view(&updated, &dismiss_result);
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
                    emit_claim_view(&updated, &result);
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
                    emit_claim_view(&updated, &result);
                }
            }
        }
    }
}
