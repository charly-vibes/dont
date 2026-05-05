use std::fs;
use std::path::{Path, PathBuf};

use crate::store::{Store, StoreError};

pub struct Project {
    pub dont_dir: PathBuf,
    pub store: Store,
}

#[derive(Debug)]
pub enum ProjectError {
    AlreadyInitialised(PathBuf),
    ConfigMissing(String),
    Store(StoreError),
    Io(std::io::Error),
}

impl std::fmt::Display for ProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyInitialised(p) => write!(f, "already initialised at {}", p.display()),
            Self::ConfigMissing(msg) => f.write_str(msg),
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

const MINIMAL_CONFIG: &str = r#"[project]
name = "dont-project"
mode = "permissive"

[output]
default_format = "json"

[trust.hedges]
patterns = ["i think", "maybe", "not sure", "probably"]

[storage]
busy_retry_attempts = 5
busy_retry_base_ms = 100
"#;

const AGENTS_MD: &str = r#"# dont

This project uses `dont` for epistemic claim tracking.

For full documentation see the [dont spec](https://github.com/charly-vibes/dont).

## Quick start

```
dont conclude "claim text"   # introduce a claim
dont trust <id> --reason ... # register doubt
dont dismiss <id> --evidence ... # verify with evidence
dont show <id>               # inspect a claim
dont list                    # list all claims
```
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
        let store = Store::open_dont_dir(&dont_dir)?;
        Ok(Self { dont_dir, store })
    }

    /// Initialize a new project. Returns `AlreadyInitialised` if `.dont/` already present.
    pub fn init(cwd: &Path) -> Result<Self, ProjectError> {
        let dont_dir = if let Ok(dir) = std::env::var("DONT_DIR") {
            PathBuf::from(dir)
        } else {
            cwd.join(".dont")
        };

        if dont_dir.join("config.toml").exists() {
            return Err(ProjectError::AlreadyInitialised(dont_dir));
        }

        fs::create_dir_all(&dont_dir)?;
        for subdir in &["seed", "vocab", "rules", "imports", "sessions", "schemas"] {
            fs::create_dir_all(dont_dir.join(subdir))?;
        }
        fs::write(dont_dir.join("config.toml"), MINIMAL_CONFIG)?;
        fs::write(dont_dir.join("AGENTS.md"), AGENTS_MD)?;

        let store = Store::open_dont_dir(&dont_dir)?;
        Ok(Self { dont_dir, store })
    }
}
