use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use serde_json::Value;

use dont::envelope::{Envelope, ErrorResult, HintEntry, RemediationEntry, Warning};
use dont::project::{Project, ProjectError};

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
    Init,

    /// Introduce an unverified claim.
    Conclude {
        /// Claim statement text.
        statement: String,
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

    /// Show a claim.
    Show {
        /// Claim identifier.
        id: String,
    },

    /// List claims.
    List,
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

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Init => {
            match Project::init(&cwd()) {
                Ok(_) => {
                    let env = Envelope::success("empty", Value::Null, vec![], vec![HintEntry {
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

        Command::Conclude { statement } => {
            let project = match Project::open(&cwd()) {
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
            };

            match project.store.append_claim(&statement) {
                Ok(result) => {
                    let payload = serde_json::json!({
                        "id": result.id,
                        "statement": statement,
                        "status": "unverified",
                        "derived_assessments": [],
                        "atoms": [],
                        "hypotheses": [],
                        "evidence": [],
                        "depends_on": [],
                        "applicable_rules": {},
                        "created_at": result.created_at,
                    });
                    let env = Envelope::success_with_tx(
                        "claim",
                        payload,
                        vec![],
                        vec![HintEntry {
                            command: format!("dont show {}", result.id),
                            description: "Inspect the new claim".to_string(),
                        }],
                        Some(result.tx as u64),
                    );
                    emit_json(&env);
                }
                Err(err) => {
                    let err_result = ErrorResult {
                        code: "internal".to_string(),
                        message: err.to_string(),
                        rule_name: None,
                        spec_ref: None,
                        entity_id: None,
                        unmet_clauses: vec![],
                        remediation: vec![RemediationEntry {
                            command: "dont doctor".to_string(),
                            description: "Run dont doctor to diagnose the issue".to_string(),
                        }],
                    };
                    emit_error_and_exit(err_result, vec![], 4);
                }
            }
        }

        Command::Trust { .. } => print_stub("trust"),
        Command::Dismiss { .. } => print_stub("dismiss"),
        Command::Show { .. } => print_stub("show"),
        Command::List => print_stub("list"),
    }
}

fn print_stub(command: &str) {
    println!("{command}: not implemented");
}
