use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use serde_json::{Value, json};

use dont::envelope::{Envelope, ErrorResult, HintEntry, RemediationEntry, Warning};
use dont::model::{
    dismiss as model_dismiss, ignore as model_ignore, reopen as model_reopen, trust as model_trust,
    Status,
};
use dont::project::{Project, ProjectError, ProjectMode};
use dont::store::{
    AppendResult, ClaimRecord, StoreError, StoreEvent, StoreEventKind, StoreStatus, TermRecord,
};

#[derive(Debug, Parser)]
#[command(name = "dont")]
#[command(version)]
#[command(about = "Epistemic forcing-function CLI for grounded claims")]
struct Cli {
    /// Output JSON envelope on stdout.
    #[arg(long, global = true)]
    json: bool,

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

        /// Reason for doubt.
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

        /// Substantive reason for ignoring (hedge-only reasons are refused).
        #[arg(long, short)]
        reason: Option<String>,
    },

    /// Show a claim.
    Show {
        /// Claim identifier.
        id: String,
    },

    /// Return session-start orientation and project state summary.
    Prime,

    /// List claims.
    List,
}

const DEFAULT_HEDGES: &[&str] = &["i think", "maybe", "not sure", "probably"];

fn contains_hedge(reason: &str) -> bool {
    let lower = reason.to_lowercase();
    DEFAULT_HEDGES.iter().any(|h| lower.contains(h))
}

fn cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn emit_json<T: serde::Serialize>(envelope: &T) {
    println!("{}", serde_json::to_string(envelope).unwrap());
}

fn emit_error_and_exit(err: ErrorResult, warnings: Vec<Warning>, code: i32) -> ! {
    let envelope = Envelope::error(err, warnings);
    emit_json(&envelope);
    process::exit(code);
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
                command: "dont doctor".to_string(),
                description: "Run dont doctor to diagnose the issue".to_string(),
            },
            RemediationEntry {
                command: "https://github.com/charly-vibes/dont/issues".to_string(),
                description: "Report the issue if dont doctor finds nothing".to_string(),
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

/// Collect all evidence URIs from all events on the claim, in event-tx order.
fn collect_evidence(record: &ClaimRecord) -> Vec<String> {
    let mut all: Vec<(i64, String)> = record
        .events
        .iter()
        .flat_map(|ev| ev.evidence.iter().map(|uri| (ev.tx, uri.clone())))
        .collect();
    all.sort_by_key(|(tx, _)| *tx);
    all.into_iter().map(|(_, uri)| uri).collect()
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
    json!({
        "id": record.id,
        "entity_kind": "claim",
        "statement": record.statement,
        "status": format!("{:?}", record.status).to_lowercase(),
        "derived_assessments": [],
        "atoms": [],
        "hypotheses": [],
        "evidence": evidence,
        "depends_on": record.depends_on,
        "applicable_rules": {},
        "created_at": record.created_at,
        "updated_at": updated_at(record),
    })
}

fn build_term_view(record: &TermRecord) -> Value {
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
        "created_at": record.created_at,
        "applicable_rules": {},
    })
}

fn refusal(code: &str, message: &str, entity_id: Option<&str>, remediation: Vec<RemediationEntry>) -> ErrorResult {
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
    let err_result = ErrorResult {
        code: "internal".to_string(),
        message: err.to_string(),
        rule_name: None,
        spec_ref: None,
        entity_id: entity_id.map(str::to_string),
        unmet_clauses: vec![],
        remediation: vec![RemediationEntry {
            command: "dont doctor".to_string(),
            description: "Run dont doctor to diagnose the issue".to_string(),
        }],
    };
    emit_error_and_exit(err_result, vec![], 4);
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
                description: format!("Prepend '{article}' to form a singular indefinite noun phrase"),
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
    let end = doc
        .find(['.', '?', '!', ';'])
        .unwrap_or(doc.len());
    let phrase = doc[..end].trim();
    let token_capped: String = phrase.split_whitespace().take(15).collect::<Vec<_>>().join(" ");
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

    match cli.command {
        Command::Init { strict } => {
            let mode = if strict {
                ProjectMode::Strict
            } else {
                ProjectMode::Permissive
            };
            match Project::init(&cwd(), mode) {
                Ok(_) => {
                    let env = Envelope::success("empty", json!({ "mode": mode.as_str() }), vec![], vec![HintEntry {
                        command: "dont conclude \"claim text\"".to_string(),
                        description: "Introduce your first claim".to_string(),
                    }]);
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

        Command::Conclude { statement, depends_on } => {
            let project = open_project_or_exit();

            let mut unresolved: Vec<String> = vec![];
            for curie in &depends_on {
                match project.store.term_curie_exists(curie) {
                    Ok(true) => {}
                    Ok(false) => unresolved.push(curie.clone()),
                    Err(err) => handle_store_error(err, None),
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
                        unresolved.iter().map(|c| RemediationEntry {
                            command: format!("dont define {c} --doc \"<definition>\""),
                            description: format!("Define the term {c} before concluding"),
                        }).collect(),
                    ),
                    vec![],
                    1,
                );
            }

            let warnings: Vec<Warning> = unresolved.iter().map(|c| Warning {
                rule_name: "unresolved-term-ref".to_string(),
                entity_id: None,
                message: format!("term reference {c} is not yet defined; verification blocked until resolved"),
                suggested_remediation: Some(format!("dont define {c} --doc \"<definition>\"")),
            }).collect();

            match project.store.append_claim(&statement, &depends_on) {
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
                        "depends_on": depends_on,
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
                            description: "Re-run with the term CURIE as the first argument".to_string(),
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
                                description: "Re-run with a concrete, non-hedged reason".to_string(),
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

        Command::Dismiss { id, evidence } => {
            if evidence.is_empty() {
                emit_error_and_exit(
                    refusal(
                        "no-evidence",
                        "dismiss requires at least one --evidence URI",
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
            let event = StoreEvent {
                kind: StoreEventKind::Dismissed,
                note: None,
                evidence: evidence.clone(),
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

        Command::Prime => {
            let project = open_project_or_exit();
            let claims = match project.store.list_claims() {
                Ok(c) => c,
                Err(err) => handle_store_error(err, None),
            };
            let mut unverified = 0;
            let mut doubted = 0;
            let mut verified = 0;
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
                    StoreStatus::Ignored | StoreStatus::Locked => {}
                }
            }
            let payload = json!({
                "project": "dont-project",
                "mode": project.mode(),
                "status_counts": {
                    "unverified": unverified,
                    "doubted": doubted,
                    "verified": verified,
                    "locked": 0,
                    "ignored": 0,
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
            let env = Envelope::success("prime", payload, vec![], vec![HintEntry {
                command: "dont help --tutorial".to_string(),
                description: "Read the first-session tutorial for the full workflow".to_string(),
            }]);
            emit_json(&env);
        }

        Command::List => {
            let project = open_project_or_exit();
            let mut claims = match project.store.list_claims() {
                Ok(c) => c,
                Err(err) => handle_store_error(err, None),
            };
            // Sort by created_at descending; use id (ULID) as tiebreaker within same second
            claims.sort_by(|a, b| {
                b.created_at
                    .cmp(&a.created_at)
                    .then_with(|| b.id.cmp(&a.id))
            });
            let views: Vec<Value> = claims.iter().map(build_claim_view).collect();
            let env = Envelope::success("claims", views, vec![], vec![]);
            emit_json(&env);
        }
    }
}
