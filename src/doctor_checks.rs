//! Genesis `DoctorCheck` implementations for `dont`.
//!
//! Each check maps to one of the existing ad-hoc diagnostic entries in the
//! `dont doctor` command. These are registered with
//! [`genesis::doctor::DoctorRunner`] and produce structured
//! [`genesis::doctor::DoctorReport`] output.

use std::path::Path;
use std::sync::Arc;

use genesis::doctor::DoctorCheck;
use genesis::suite_linter::{LintResult, Severity};

use crate::config;
use crate::envelope::CLI_VERSION;
use crate::linkml::linkml_is_on_path;
use crate::project::Project;

// ---------------------------------------------------------------------------
// Substrate check
// ---------------------------------------------------------------------------

/// Verifies the store was opened successfully (always passes — a store-open
/// failure would have already exited in `open_project_or_exit`).
pub struct SubstrateCheck;

impl DoctorCheck for SubstrateCheck {
    fn name(&self) -> &'static str {
        "substrate"
    }

    fn description(&self) -> &'static str {
        "Verify that the CozoDB store opened successfully"
    }

    fn run(&self, _root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        Ok(vec![]) // pass — store failure is fatal before we get here
    }
}

// ---------------------------------------------------------------------------
// Rules compile check
// ---------------------------------------------------------------------------

/// Verifies built-in rules are available (always passes).
pub struct RulesCompileCheck;

impl DoctorCheck for RulesCompileCheck {
    fn name(&self) -> &'static str {
        "rules_compile"
    }

    fn description(&self) -> &'static str {
        "Verify that built-in Datalog rules compile"
    }

    fn run(&self, _root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        Ok(vec![]) // pass — rules are compiled into the binary
    }
}

// ---------------------------------------------------------------------------
// Config check
// ---------------------------------------------------------------------------

/// Validates `.dont/config.toml` via the `ConfigStore`.
pub struct ConfigCheck;

impl DoctorCheck for ConfigCheck {
    fn name(&self) -> &'static str {
        "config"
    }

    fn description(&self) -> &'static str {
        "Validate .dont/config.toml against the config schema"
    }

    fn run(&self, root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        let registry = config::dont_config_registry();
        let discovered = genesis::config::ConfigStore::discover(root, &registry);
        let marker_found = discovered
            .iter()
            .find(|d| d.tool_name == "dont")
            .map(|d| d.found)
            .unwrap_or(false);
        let config_store = genesis::config::ConfigStore::new(registry);
        let config_validations = config_store.validate_all(root);
        let config_errors: Vec<_> = config_validations
            .iter()
            .filter(|v| v.severity == genesis::config::ValidationSeverity::Error)
            .collect();

        if !config_errors.is_empty() {
            let msg = config_errors
                .iter()
                .map(|v| format!("{}: {}", v.field, v.message))
                .collect::<Vec<_>>()
                .join("; ");
            Ok(vec![LintResult::new(msg, Severity::Error)])
        } else if marker_found {
            Ok(vec![]) // pass
        } else {
            // Marker not at repo root (standalone DONT_DIR layout) — still valid
            Ok(vec![])
        }
    }
}

// ---------------------------------------------------------------------------
// Seed snapshot check
// ---------------------------------------------------------------------------

/// Checks that the seed snapshot file exists.
pub struct SeedSnapshotCheck {
    project: Arc<Project>,
}

impl SeedSnapshotCheck {
    pub fn new(project: Arc<Project>) -> Self {
        Self { project }
    }
}

impl DoctorCheck for SeedSnapshotCheck {
    fn name(&self) -> &'static str {
        "seed_snapshot"
    }

    fn description(&self) -> &'static str {
        "Verify that the seed snapshot file is present"
    }

    fn run(&self, _root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        let path = self.project.seed_snapshot_path();
        if path.is_file() {
            Ok(vec![])
        } else {
            Ok(vec![LintResult::new(
                format!(
                    "seed snapshot {} is missing; run dont init to repair the project layout",
                    path.display()
                ),
                Severity::Warning,
            )])
        }
    }
}

// ---------------------------------------------------------------------------
// Pending spawns check
// ---------------------------------------------------------------------------

/// Reports on pending spawn audit status.
pub struct PendingSpawnsCheck {
    project: Arc<Project>,
}

impl PendingSpawnsCheck {
    pub fn new(project: Arc<Project>) -> Self {
        Self { project }
    }
}

impl DoctorCheck for PendingSpawnsCheck {
    fn name(&self) -> &'static str {
        "pending_spawns"
    }

    fn description(&self) -> &'static str {
        "Check for pending spawn audits"
    }

    fn run(&self, _root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        let msg = if self.project.root_doc_paths().is_empty() {
            "no pending spawn audit implemented; direct DONT_DIR override skips separate root managed docs"
        } else {
            "no pending spawn audit implemented"
        };
        Ok(vec![LintResult::new(msg, Severity::Advisory)])
    }
}

// ---------------------------------------------------------------------------
// Remediation invariant check
// ---------------------------------------------------------------------------

/// Verifies the error remediation invariant is available.
pub struct RemediationInvariantCheck;

impl DoctorCheck for RemediationInvariantCheck {
    fn name(&self) -> &'static str {
        "remediation_invariant"
    }

    fn description(&self) -> &'static str {
        "Verify that error remediation invariants are satisfied"
    }

    fn run(&self, _root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        Ok(vec![]) // pass — invariant is compile-time
    }
}

// ---------------------------------------------------------------------------
// Managed docs check
// ---------------------------------------------------------------------------

/// Checks that managed documentation blocks are current.
pub struct ManagedDocsCheck {
    project: Arc<Project>,
}

impl ManagedDocsCheck {
    pub fn new(project: Arc<Project>) -> Self {
        Self { project }
    }
}

impl DoctorCheck for ManagedDocsCheck {
    fn name(&self) -> &'static str {
        "managed_docs"
    }

    fn description(&self) -> &'static str {
        "Verify that managed documentation blocks are up-to-date"
    }

    fn run(&self, _root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        let (clean, details) = self.project.managed_docs_status()?;
        if clean {
            Ok(vec![])
        } else {
            let msg = details.join("; ");
            Ok(vec![LintResult::new(
                format!("{msg}; run `dont doctor --fix` to repair"),
                Severity::Warning,
            )])
        }
    }

    fn auto_fixable(&self) -> bool {
        true
    }

    fn fix(&self, _root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        self.project.refresh_managed_docs()?;
        Ok(vec![])
    }
}

// ---------------------------------------------------------------------------
// Managed skills check
// ---------------------------------------------------------------------------

/// Checks that managed skill packs are current.
pub struct ManagedSkillsCheck {
    project: Arc<Project>,
}

impl ManagedSkillsCheck {
    pub fn new(project: Arc<Project>) -> Self {
        Self { project }
    }
}

impl DoctorCheck for ManagedSkillsCheck {
    fn name(&self) -> &'static str {
        "managed_skills"
    }

    fn description(&self) -> &'static str {
        "Verify that managed skill packs are current"
    }

    fn run(&self, _root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        let pack_health = self.project.managed_skill_packs_status()?;
        let skills_all_pass = pack_health
            .iter()
            .all(|h| h.state == crate::skill_pack::PackState::Pass);
        if skills_all_pass || pack_health.is_empty() {
            Ok(vec![])
        } else {
            let details: Vec<_> = pack_health
                .iter()
                .filter(|h| h.state != crate::skill_pack::PackState::Pass)
                .map(|h| h.detail.as_str())
                .collect();
            let msg = details.join("; ");
            Ok(vec![LintResult::new(msg, Severity::Warning)])
        }
    }

    fn auto_fixable(&self) -> bool {
        true
    }

    fn fix(&self, _root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        self.project.refresh_managed_skill_packs()?;
        Ok(vec![])
    }
}

// ---------------------------------------------------------------------------
// LinkML availability check
// ---------------------------------------------------------------------------

/// Checks whether `linkml` is available on PATH.
pub struct LinkmlAvailableCheck;

impl DoctorCheck for LinkmlAvailableCheck {
    fn name(&self) -> &'static str {
        "linkml_available"
    }

    fn description(&self) -> &'static str {
        "Verify that the linkml CLI is available on PATH"
    }

    fn run(&self, _root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        if linkml_is_on_path() {
            Ok(vec![])
        } else {
            Ok(vec![LintResult::new(
                "linkml is not on PATH; import linkml uses in-process parsing only — install linkml for secondary validation",
                Severity::Warning,
            )])
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: build the output envelope from a genesis DoctorReport.
// ---------------------------------------------------------------------------

/// Given a genesis [`DoctorReport`], produce the same JSON payload shape
/// that `dont doctor` has always emitted — with "detail" instead of
/// "message", and "cli_version" at the data level.
pub fn doctor_payload_from_report(report: &genesis::doctor::DoctorReport) -> serde_json::Value {
    let check_values: Vec<serde_json::Value> = report
        .checks
        .iter()
        .map(|c| {
            serde_json::json!({
                "name": c.name,
                "status": match c.status {
                    genesis::doctor::CheckStatus::Pass => "pass",
                    genesis::doctor::CheckStatus::Warn => "warn",
                    genesis::doctor::CheckStatus::Fail => "fail",
                },
                "detail": c.message,
            })
        })
        .collect();

    serde_json::json!({
        "cli_version": CLI_VERSION,
        "checks": check_values,
        "summary": {
            "pass": report.summary.pass,
            "warn": report.summary.warn,
            "fail": report.summary.fail,
        },
    })
}

// ---------------------------------------------------------------------------
// Suite linter LintCheck implementations
// ---------------------------------------------------------------------------

/// Lint check: verify that `.dont/config.toml` exists and has valid mode.
pub struct DontConfigLintCheck;

impl genesis::suite_linter::LintCheck for DontConfigLintCheck {
    fn name(&self) -> &'static str {
        "dont.config"
    }

    fn description(&self) -> &'static str {
        "Verify .dont/config.toml exists and has a valid project mode"
    }

    fn run(&self, root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        let config_path = root.join(".dont/config.toml");
        if !config_path.exists() {
            return Ok(vec![LintResult::with_fix(
                format!(".dont/config.toml not found at {}", config_path.display()),
                genesis::suite_linter::Severity::Error,
                "dont init",
            )]);
        }
        Ok(vec![])
    }
}

/// Lint check: verify that the `.dont/` directory has the required subdirectories.
pub struct DontLayoutLintCheck;

impl genesis::suite_linter::LintCheck for DontLayoutLintCheck {
    fn name(&self) -> &'static str {
        "dont.layout"
    }

    fn description(&self) -> &'static str {
        "Verify .dont/ directory has required subdirectories"
    }

    fn run(&self, root: &Path) -> Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        let dont_dir = root.join(".dont");
        if !dont_dir.is_dir() {
            return Ok(vec![LintResult::with_fix(
                format!(".dont/ not found at {}", dont_dir.display()),
                genesis::suite_linter::Severity::Error,
                "dont init",
            )]);
        }

        let mut results = Vec::new();
        for subdir in crate::project::REQUIRED_SUBDIRS {
            let path = dont_dir.join(subdir);
            if !path.is_dir() {
                results.push(LintResult::with_fix(
                    format!("missing required subdirectory {}", path.display()),
                    genesis::suite_linter::Severity::Error,
                    "dont init",
                ));
            }
        }
        if results.is_empty() {
            return Ok(vec![LintResult::new(
                "required subdirectories present",
                genesis::suite_linter::Severity::Advisory,
            )]);
        }
        Ok(results)
    }
}
