use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;

use crate::config::{Config, ConfigValidationError};
use crate::fs_util::write_restricted;
use crate::managed_block::{
    file_matches, render_root_block, replace_or_prepend_root_block, root_block_matches,
    write_canonical,
};
use crate::skill_pack;

use chrono::{SecondsFormat, Utc};
use serde_json::json;
use toml;

use crate::store::{Store, StoreError};

pub const REQUIRED_SUBDIRS: &[&str] = &["seed", "vocab", "rules", "imports", "sessions", "schemas"];

pub struct Project {
    pub dont_dir: PathBuf,
    pub store: Store,
}

#[derive(Debug)]
pub enum ProjectError {
    AlreadyInitialised(PathBuf),
    ConfigMissing(String),
    ConfigInvalid(String),
    LayoutInvalid(PathBuf),
    Store(StoreError),
    Io {
        op: Option<&'static str>,
        path: Option<PathBuf>,
        source: std::io::Error,
    },
}

impl std::fmt::Display for ProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyInitialised(p) => write!(f, "already initialised at {}", p.display()),
            Self::ConfigMissing(msg) => f.write_str(msg),
            Self::ConfigInvalid(msg) => f.write_str(msg),
            Self::LayoutInvalid(p) => write!(
                f,
                "project layout is corrupt: {} is missing or not a directory — run 'dont init' to repair",
                p.display()
            ),
            Self::Store(e) => write!(f, "store error: {e}"),
            Self::Io { op, path, source } => match (op, path) {
                (Some(op), Some(path)) => write!(f, "failed to {op} {}: {source}", path.display()),
                _ => write!(f, "I/O error: {source}"),
            },
        }
    }
}

impl std::error::Error for ProjectError {}

impl From<StoreError> for ProjectError {
    fn from(e: StoreError) -> Self {
        Self::Store(e)
    }
}

impl From<std::io::Error> for ProjectError {
    fn from(e: std::io::Error) -> Self {
        Self::Io {
            op: None,
            path: None,
            source: e,
        }
    }
}

fn io_error(op: &'static str, path: impl Into<PathBuf>, source: std::io::Error) -> ProjectError {
    ProjectError::Io {
        op: Some(op),
        path: Some(path.into()),
        source,
    }
}

fn minimal_config(mode: ProjectMode) -> String {
    format!(
        r#"[project]
name = "dont-project"
mode = "{}"

[output]
default_format = "json"

[storage]
busy_retry_attempts = 5
busy_retry_base_ms = 100

[harness]
managed_docs = ["AGENTS.md", "CLAUDE.md"]
spawn_timeout_hours = 24
"#,
        mode.as_str()
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectMode {
    Permissive,
    Strict,
}

impl ProjectMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Permissive => "permissive",
            Self::Strict => "strict",
        }
    }
}

const SEED_VOCABULARY: &str = r#"# dont seed vocabulary snapshot
# Installed project-locally by `dont init`. Tool upgrades MUST NOT rewrite this
# file except through an explicit seed-migration operation.
prefix: "dont:"
terms:
  - id: "dont:Entity"
    status: locked
  - id: "dont:Claim"
    status: locked
  - id: "dont:Term"
    status: locked
  - id: "dont:Evidence"
    status: locked
  - id: "dont:kind_of"
    status: locked
  - id: "dont:related_to"
    status: locked
  - id: "dont:defined_as"
    status: locked
  - id: "dont:Hypothesis"
    status: locked
  - id: "dont:Retraction"
    status: locked
  - id: "dont:external_ref"
    status: locked
"#;

const AGENTS_MD_BODY: &str = r#"# dont

This project uses `dont` for epistemic claim tracking.

For full documentation see the [dont spec](https://github.com/charly-vibes/dont).

## Quick start

```
dont ground "claim text" --file README.md --lines 10-18 # fast path: claim + repo evidence
dont conclude "claim text"                            # introduce an unverified claim
dont trust <id> --reason ...                          # register doubt
dont dismiss <id> --evidence ...                      # verify with evidence
dont show <id>                                        # inspect a claim
dont trace <id>                                       # diagnose blocker paths
dont list                                             # list all claims
```

`dont ground` is the preferred fast path when you already have the claim and evidence in hand. The underlying model is still conclude → trust → dismiss → forget: `ground` composes `conclude` and `dismiss` rather than bypassing the core lifecycle.

## Grounding claims in repository evidence

Prefer repository-relative file locators over opaque `file://` URIs when the evidence lives inside the current project:

```
# Preferred: repository-relative locator
dont ground "documented project fact" --file README.md --lines 10-18
dont dismiss <id> --file src/lib.rs --lines 42-55 --anchor "MyTrait"
dont dismiss <id> --file docs/spec.md --excerpt "The system SHALL..."

# Supported for compatibility: plain URI
dont dismiss <id> --evidence https://external-source.example/ref
```

Repository-relative locators resolve against the project root regardless of the caller's working directory. Paths that escape the project root (via `..` traversal or symlink escape) are refused.

When `show` or `why` reports stale, unresolved, or otherwise confusing blockers, run `dont trace <id>` to see the blocker path that explains what dependency or support fallout needs attention.

## Modes

`dont` operates in two modes:

- **permissive** — allows claims to progress with weaker evidence; suitable for exploratory work.
- **strict** — enforces full evidence and lifecycle requirements before dismissal.

Run `dont prime --json` to see the current mode. To change mode, edit `mode = "permissive"` or `mode = "strict"` in `.dont/config.toml`.

## Error recovery

When a command returns `"ok": false`, read `data.remediation[0].command` from the JSON envelope and run it exactly as printed. Do not guess reformulations — the remediation field is authoritative.

## Defining terms

Before running `dont define`, run `dont suggest-term "<description>"` to check for an existing term that fits. When defining, always supply both `--label "<a noun phrase>"` and `--doc "<definition text>"` together.

## Spawn

When `dont` requests a sub-agent via a spawn envelope, the harness is responsible for fulfilling the spawn. Run the spawn command from the envelope's `data.command` field and feed its output back to the calling agent.

## Help

Run `dont help --tutorial` for the first-session walkthrough.
Run `dont help --topics` to list all how-to guides.
Run `dont help --howto <topic>` to read a specific guide.
"#;

/// Resolves the `.dont` directory for a command invocation.
///
/// Priority:
/// 1. `DONT_DIR` environment variable (test-isolation and direct override).
/// 2. Walk up from `cwd` looking for a `.dont/` directory.
pub fn resolve_dont_dir(cwd: &Path) -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("DONT_DIR") {
        return Some(PathBuf::from(dir));
    }
    find_dont_dir_by_walking(cwd)
}

pub fn project_root_from_dont_dir(dont_dir: &Path) -> PathBuf {
    if dont_dir.file_name().and_then(|name| name.to_str()) == Some(".dont") {
        dont_dir
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| dont_dir.to_path_buf())
    } else {
        dont_dir.to_path_buf()
    }
}

fn find_dont_dir_by_walking(start: &Path) -> Option<PathBuf> {
    let mut current = start;
    loop {
        let candidate = current.join(".dont");
        if candidate.is_dir() {
            return Some(candidate);
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => return None,
        }
    }
}

fn canonical_agents_content() -> String {
    format!(
        "# dont managed instructions\n\n> This file is managed by `dont init` and `dont doctor --fix`. Do not edit manually.\n\n{AGENTS_MD_BODY}\n"
    )
}

fn root_block_content() -> String {
    render_root_block(
        "# DONT MANAGED BLOCK — DO NOT EDIT\n\nThis project uses `dont` for grounded-claim workflow.\n\nAt session start run `dont prime --json`.\n\nCanonical agent instructions: `.dont/AGENTS.md`.\n\nEdits inside this managed block will be overwritten by `dont doctor --fix`.",
    )
}

impl Project {
    /// Open an existing project. Returns `ConfigMissing` if no `.dont/` found.
    pub fn open(cwd: &Path) -> Result<Self, ProjectError> {
        let dont_dir = resolve_dont_dir(cwd).ok_or_else(|| {
            ProjectError::ConfigMissing(
                "no .dont/ project found — run 'dont init' first".to_string(),
            )
        })?;
        if !dont_dir.join("config.toml").exists() {
            return Err(ProjectError::ConfigMissing(format!(
                "project at {} is missing config.toml — run 'dont init' to repair",
                dont_dir.display()
            )));
        }
        for subdir in REQUIRED_SUBDIRS {
            let path = dont_dir.join(subdir);
            if !path.is_dir() {
                return Err(ProjectError::LayoutInvalid(path));
            }
        }
        let store = Store::open_dont_dir(&dont_dir)?;
        Ok(Self { dont_dir, store })
    }

    pub fn mode(&self) -> Option<ProjectMode> {
        let raw = self.load_config().project.mode?;
        match raw.as_str() {
            "permissive" => Some(ProjectMode::Permissive),
            "strict" => Some(ProjectMode::Strict),
            _ => None,
        }
    }

    pub fn load_config(&self) -> Config {
        let text = fs::read_to_string(self.dont_dir.join("config.toml")).unwrap_or_default();
        toml::from_str(&text).unwrap_or_default()
    }

    /// Load config.toml and validate all constrained fields. Returns
    /// `ConfigInvalid` if any field holds an out-of-range value so the
    /// caller can surface a structured error before any command logic runs.
    ///
    /// Unlike `load_config`, this variant propagates TOML parse errors as
    /// `ConfigInvalid` so that a malformed config.toml is surfaced to the
    /// caller rather than silently replaced with defaults.
    pub fn load_validated_config(&self) -> Result<Config, ProjectError> {
        let text = fs::read_to_string(self.dont_dir.join("config.toml")).unwrap_or_default();
        let config: Config = toml::from_str(&text)
            .map_err(|e| ProjectError::ConfigInvalid(format!("config.toml: {e}")))?;
        config
            .validate()
            .map_err(|e: ConfigValidationError| ProjectError::ConfigInvalid(e.message))?;
        Ok(config)
    }

    pub fn project_root(&self) -> PathBuf {
        project_root_from_dont_dir(&self.dont_dir)
    }

    fn uses_separate_root_docs(&self) -> bool {
        // Compatibility path: many tests and direct overrides point DONT_DIR at a standalone
        // state directory rather than a real `<project>/.dont` folder. In that mode the
        // canonical AGENTS file would collide with a root AGENTS.md, so we only manage root
        // documents when the state directory is literally named `.dont`.
        self.dont_dir.file_name().and_then(|name| name.to_str()) == Some(".dont")
    }

    pub fn seed_snapshot_path(&self) -> PathBuf {
        self.dont_dir.join("seed/dont-seed.yaml")
    }

    pub fn canonical_agents_path(&self) -> PathBuf {
        self.dont_dir.join("AGENTS.md")
    }

    pub fn root_doc_paths(&self) -> Vec<PathBuf> {
        if !self.uses_separate_root_docs() {
            return vec![];
        }
        let root = self.project_root();
        self.load_config()
            .harness
            .managed_docs
            .into_iter()
            .map(|doc| root.join(doc))
            .collect()
    }

    pub fn refresh_managed_docs(&self) -> Result<(), ProjectError> {
        let canonical = canonical_agents_content();
        let canonical_path = self.canonical_agents_path();
        write_canonical(&canonical_path, &canonical)
            .map_err(|err| io_error("write", &canonical_path, err))?;
        let root_block = root_block_content();
        for path in self.root_doc_paths() {
            replace_or_prepend_root_block(&path, &root_block)
                .map_err(|err| io_error("write", &path, err))?;
        }
        Ok(())
    }

    pub fn managed_docs_status(&self) -> Result<(bool, Vec<String>), ProjectError> {
        let mut clean = true;
        let mut details = Vec::new();
        let canonical = canonical_agents_content();
        if !file_matches(&self.canonical_agents_path(), &canonical)? {
            clean = false;
            details.push(format!(
                "canonical file {} is stale; run dont doctor --fix",
                self.canonical_agents_path().display()
            ));
        }
        let root_block = root_block_content();
        for path in self.root_doc_paths() {
            if !root_block_matches(&path, &root_block)? {
                clean = false;
                details.push(format!(
                    "managed block in {} is missing or stale; run dont doctor --fix",
                    path.display()
                ));
            }
        }
        Ok((clean, details))
    }

    fn skills_dir(&self) -> PathBuf {
        self.project_root().join(".agents").join("skills")
    }

    fn configured_skill_packs(&self) -> Vec<String> {
        self.load_config().harness.managed_skill_packs
    }

    pub fn refresh_managed_skill_packs(&self) -> Result<(), ProjectError> {
        let skills_root = self.skills_dir();
        for pack_name in self.configured_skill_packs() {
            let files = skill_pack::generate_pack(&pack_name).map_err(|msg| {
                ProjectError::ConfigInvalid(format!("managed_skill_packs: {msg}"))
            })?;
            let pack_dir = skills_root.join(&pack_name);
            // Remove the entire pack dir and recreate it so that files removed from
            // the generator in a future version don't linger and cause an infinite
            // stale loop (disk hash would never match the generated set).
            if pack_dir.exists() {
                fs::remove_dir_all(&pack_dir).map_err(|e| io_error("remove", &pack_dir, e))?;
            }
            fs::create_dir_all(&pack_dir).map_err(|e| io_error("create", &pack_dir, e))?;
            for (rel, content) in &files {
                let dest = pack_dir.join(rel);
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).map_err(|e| io_error("create", parent, e))?;
                }
                write_canonical(&dest, content).map_err(|e| io_error("write", &dest, e))?;
            }
        }
        Ok(())
    }

    pub fn managed_skill_packs_status(&self) -> Result<Vec<skill_pack::PackHealth>, ProjectError> {
        let skills_root = self.skills_dir();
        let mut results = Vec::new();
        for pack_name in self.configured_skill_packs() {
            let pack_dir = skills_root.join(&pack_name);
            let generated = skill_pack::generate_pack(&pack_name).map_err(|msg| {
                ProjectError::ConfigInvalid(format!("managed_skill_packs: {msg}"))
            })?;
            let expected_hash = skill_pack::pack_content_hash(&generated);
            let health = if !pack_dir.exists()
                || pack_dir.read_dir().map_or(true, |mut d| d.next().is_none())
            {
                skill_pack::PackHealth {
                    name: pack_name.clone(),
                    state: skill_pack::PackState::Missing,
                    detail: format!("managed pack {pack_name} is missing; run dont doctor --fix"),
                }
            } else {
                let disk_hash = skill_pack::disk_content_hash(&pack_dir)
                    .map_err(|e| io_error("read", &pack_dir, e))?;
                if disk_hash != expected_hash {
                    skill_pack::PackHealth {
                        name: pack_name.clone(),
                        state: skill_pack::PackState::Stale,
                        detail: format!("managed pack {pack_name} is stale; run dont doctor --fix"),
                    }
                } else {
                    skill_pack::PackHealth {
                        name: pack_name.clone(),
                        state: skill_pack::PackState::Pass,
                        detail: format!("managed pack {pack_name} is current"),
                    }
                }
            };
            results.push(health);
        }
        Ok(results)
    }

    /// Detect if the config.toml mode differs from the last recorded mode in events.jsonl
    /// and append a `mode.changed` event if so.
    pub fn check_and_record_mode_change(&self) {
        let Some(current_mode) = self.mode() else {
            return;
        };
        let current_mode = current_mode.as_str();
        let events_path = self.dont_dir.join("events.jsonl");
        let events_text = fs::read_to_string(&events_path).unwrap_or_default();
        let last_recorded_mode = events_text
            .lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .filter_map(|v| {
                v.get("mode")
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string())
            })
            .next_back();
        let Some(recorded) = last_recorded_mode else {
            // No mode event found — project predates mode tracking. Write a baseline
            // so future changes are detectable without false-positives.
            let created_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
            let event = json!({
                "kind": "mode.baseline",
                "mode": current_mode,
                "created_at": created_at,
            });
            let line = format!("{}\n", serde_json::to_string(&event).unwrap_or_default());
            if let Err(e) = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&events_path)
                .and_then(|mut f| {
                    use std::io::Write;
                    f.write_all(line.as_bytes())
                })
            {
                eprintln!("dont: warning: could not write mode baseline event: {e}");
            }
            return;
        };
        if recorded != current_mode {
            let created_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
            let event = json!({
                "kind": "mode.changed",
                "from": recorded,
                "to": current_mode,
                "mode": current_mode,
                "created_at": created_at,
            });
            let line = format!("{}\n", serde_json::to_string(&event).unwrap_or_default());
            if let Err(e) = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&events_path)
                .and_then(|mut f| {
                    use std::io::Write;
                    f.write_all(line.as_bytes())
                })
            {
                eprintln!("dont: warning: could not write mode.changed event: {e}");
            }
        }
    }

    /// Initialize a new project. Returns `AlreadyInitialised` if `.dont/` already present.
    pub fn init(cwd: &Path, mode: ProjectMode) -> Result<Self, ProjectError> {
        let dont_dir = if let Ok(dir) = std::env::var("DONT_DIR") {
            PathBuf::from(dir)
        } else {
            cwd.join(".dont")
        };

        if dont_dir.join("config.toml").exists() {
            return Err(ProjectError::AlreadyInitialised(dont_dir));
        }

        mkdir_restricted(&dont_dir).map_err(|err| io_error("create", &dont_dir, err))?;
        for subdir in REQUIRED_SUBDIRS {
            let path = dont_dir.join(subdir);
            mkdir_restricted(&path).map_err(|err| io_error("create", &path, err))?;
        }
        let config_path = dont_dir.join("config.toml");
        write_restricted(&config_path, minimal_config(mode).as_bytes())
            .map_err(|err| io_error("write", &config_path, err))?;
        let seed_path = dont_dir.join("seed/dont-seed.yaml");
        write_restricted(&seed_path, SEED_VOCABULARY.as_bytes())
            .map_err(|err| io_error("write", &seed_path, err))?;
        let events_path = dont_dir.join("events.jsonl");
        write_restricted(&events_path, init_event(mode).as_bytes())
            .map_err(|err| io_error("write", &events_path, err))?;
        if std::env::var("DONT_DIR").is_err() {
            ensure_dont_gitignore_entry(cwd)?;
        }

        let store = Store::open_dont_dir(&dont_dir)?;
        let project = Self { dont_dir, store };
        project.refresh_managed_docs()?;
        project.refresh_managed_skill_packs()?;
        Ok(project)
    }
}

fn init_event(mode: ProjectMode) -> String {
    let created_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    format!(
        "{{\"kind\":\"project.initialized\",\"mode\":\"{}\",\"created_at\":\"{}\"}}\n",
        mode.as_str(),
        created_at,
    )
}

/// Create `path` as a directory (recursively) with mode 0o700 on Unix.
fn mkdir_restricted(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)?;
    #[cfg(not(unix))]
    fs::create_dir_all(path)?;
    Ok(())
}

fn ensure_dont_gitignore_entry(project_root: &Path) -> Result<(), ProjectError> {
    let path = project_root.join(".gitignore");
    let mut content = fs::read_to_string(&path).unwrap_or_default();
    if content.lines().any(|line| line.trim() == ".dont/") {
        return Ok(());
    }
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(".dont/\n");
    fs::write(&path, content).map_err(|err| io_error("write", &path, err))?;
    Ok(())
}
