use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;

use chrono::{SecondsFormat, Utc};
use toml;
use serde_json::json;

use crate::store::{Store, StoreError};

pub const REQUIRED_SUBDIRS: &[&str] =
    &["seed", "vocab", "rules", "imports", "sessions", "schemas"];

pub struct Project {
    pub dont_dir: PathBuf,
    pub store: Store,
}

#[derive(Debug)]
pub enum ProjectError {
    AlreadyInitialised(PathBuf),
    ConfigMissing(String),
    LayoutInvalid(PathBuf),
    Store(StoreError),
    Io(std::io::Error),
}

impl std::fmt::Display for ProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyInitialised(p) => write!(f, "already initialised at {}", p.display()),
            Self::ConfigMissing(msg) => f.write_str(msg),
            Self::LayoutInvalid(p) => write!(
                f,
                "project layout is corrupt: {} is missing or not a directory — run 'dont init' to repair",
                p.display()
            ),
            Self::Store(e) => write!(f, "store error: {e}"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
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
        Self::Io(e)
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

const AGENTS_MD: &str = r#"# dont

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

    pub fn mode(&self) -> String {
        let config = fs::read_to_string(self.dont_dir.join("config.toml")).unwrap_or_default();
        config
            .lines()
            .find_map(|line| {
                let line = line.trim();
                line.strip_prefix("mode = ")
                    .map(|value| value.trim_matches('"').to_string())
            })
            .unwrap_or_else(|| "unknown".to_string())
    }

    pub fn load_config(&self) -> Config {
        let text = fs::read_to_string(self.dont_dir.join("config.toml")).unwrap_or_default();
        toml::from_str(&text).unwrap_or_default()
    }

    /// Detect if the config.toml mode differs from the last recorded mode in events.jsonl
    /// and append a `mode.changed` event if so.
    pub fn check_and_record_mode_change(&self) {
        let current_mode = self.mode();
        if current_mode == "unknown" {
            return;
        }
        let events_path = self.dont_dir.join("events.jsonl");
        let events_text = fs::read_to_string(&events_path).unwrap_or_default();
        let last_recorded_mode = events_text
            .lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .filter_map(|v| v.get("mode").and_then(|m| m.as_str()).map(|s| s.to_string()))
            .last();
        if let Some(recorded) = last_recorded_mode {
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
                let _ = fs::OpenOptions::new().append(true).open(&events_path)
                    .and_then(|mut f| { use std::io::Write; f.write_all(line.as_bytes()) });
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

        fs::create_dir_all(&dont_dir)?;
        for subdir in REQUIRED_SUBDIRS {
            fs::create_dir_all(dont_dir.join(subdir))?;
        }
        fs::write(dont_dir.join("config.toml"), minimal_config(mode))?;
        fs::write(dont_dir.join("AGENTS.md"), AGENTS_MD)?;
        fs::write(dont_dir.join("seed/dont-seed.yaml"), SEED_VOCABULARY)?;
        fs::write(dont_dir.join("events.jsonl"), init_event(mode))?;
        if std::env::var("DONT_DIR").is_err() {
            ensure_dont_gitignore_entry(cwd)?;
        }

        let store = Store::open_dont_dir(&dont_dir)?;
        Ok(Self { dont_dir, store })
    }
}

fn init_event(mode: ProjectMode) -> String {
    let created_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let event = json!({
        "kind": "project.initialized",
        "mode": mode.as_str(),
        "created_at": created_at,
    });
    format!("{}\n", serde_json::to_string(&event).expect("project init event serializes"))
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
    fs::write(path, content)?;
    Ok(())
}
