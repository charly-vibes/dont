use std::fmt;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};

use crate::fs_util::write_restricted;
pub use crate::model::Status;

fn parse_status(value: &str) -> Result<Status, StoreError> {
    Status::from_persisted_str(value)
        .ok_or_else(|| StoreError::Malformed(format!("unknown status {value}")))
}

use chrono::{SecondsFormat, Utc};
use cozo::DbInstance;
use fs2::FileExt;
use serde_json::Value;
use sha2::{Digest, Sha256};
use ulid::Ulid;

const SCHEMA_VERSION: i64 = 1;

pub struct Store {
    db: DbInstance,
    path: PathBuf,
    lock_path: PathBuf,
    seq_path: PathBuf,
}

impl fmt::Debug for Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Store").field("path", &self.path).finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendResult {
    pub id: String,
    pub event_id: String,
    pub tx: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreEventKind {
    Concluded,
    Defined,
    Trusted,
    Flagged,
    Undoubted,
    Locked,
    Ignored,
    Reopened,
    HypothesisAdded,
    HypothesisAssessed,
    AtomDefined,
    AtomDismissed,
}

impl StoreEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Concluded => "concluded",
            Self::Defined => "defined",
            Self::Trusted => "trusted",
            Self::Flagged => "flagged",
            Self::Undoubted => "undoubted",
            Self::Locked => "locked",
            Self::Ignored => "ignored",
            Self::Reopened => "reopened",
            Self::HypothesisAdded => "hypothesis-added",
            Self::HypothesisAssessed => "hypothesis-assessed",
            Self::AtomDefined => "atom-defined",
            Self::AtomDismissed => "atom-dismissed",
        }
    }

    fn from_str(value: &str) -> Result<Self, StoreError> {
        match value {
            "concluded" => Ok(Self::Concluded),
            "defined" => Ok(Self::Defined),
            "trusted" => Ok(Self::Trusted),
            "flagged" | "dismissed" => Ok(Self::Flagged),
            "undoubted" => Ok(Self::Undoubted),
            "locked" => Ok(Self::Locked),
            "ignored" => Ok(Self::Ignored),
            "reopened" => Ok(Self::Reopened),
            "hypothesis-added" => Ok(Self::HypothesisAdded),
            "hypothesis-assessed" => Ok(Self::HypothesisAssessed),
            "atom-defined" => Ok(Self::AtomDefined),
            "atom-dismissed" => Ok(Self::AtomDismissed),
            _ => Err(StoreError::Malformed(format!("unknown event kind {value}"))),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StoreEvent {
    pub kind: StoreEventKind,
    pub note: Option<String>,
    pub evidence: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Datom {
    pub entity: String,
    pub attribute: String,
    pub value: Value,
    pub tx: i64,
    pub assert_bit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AtomRecord {
    pub idx: usize,
    pub text: String,
    pub status: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HypothesisAssessment {
    pub supporting: Vec<String>,
    pub refuting: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HypothesisRecord {
    pub idx: usize,
    pub text: String,
    pub assessment: HypothesisAssessment,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClaimRecord {
    pub id: String,
    pub statement: String,
    pub status: Status,
    pub depends_on: Vec<String>,
    pub atoms: Vec<AtomRecord>,
    pub hypotheses: Vec<HypothesisRecord>,
    pub created_at: String,
    pub events: Vec<EventRecord>,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TermRecord {
    pub id: String,
    pub curie: String,
    pub label: Option<String>,
    pub definition: String,
    pub status: Status,
    pub created_at: String,
    pub events: Vec<EventRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedTermRecord {
    pub id: String,
    pub curie: String,
    pub label: Option<String>,
    pub definition: String,
    pub source: String,
    pub imported_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CurieResolution {
    Coined(TermRecord),
    Imported(ImportedTermRecord),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventRecord {
    pub id: String,
    pub kind: StoreEventKind,
    pub note: Option<String>,
    pub evidence: Vec<Value>,
    pub tx: i64,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct TrustEventRow {
    pub event_id: String,
    pub claim_id: String,
    pub kind: String,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug)]
pub enum StoreError {
    Io(std::io::Error),
    Cozo(String),
    /// The backing store file is corrupt or unreadable.
    ///
    /// `path` names the file that triggered the error.
    /// `detail` is the underlying message from the storage engine or OS.
    CorruptStore {
        path: PathBuf,
        detail: String,
    },
    SchemaMismatch {
        found: i64,
        expected: i64,
    },
    CurieConflict {
        curie: String,
        existing_id: String,
    },
    AmbiguousPrefix {
        prefix: String,
        candidates: Vec<String>,
    },
    Malformed(String),
    DuplicateClaim {
        text_hash: String,
        existing_id: String,
    },
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Cozo(message) => write!(f, "Cozo error: {message}"),
            Self::CorruptStore { path, detail } => write!(
                f,
                "store file {} is corrupt or unreadable: {}; delete or restore the file and re-run 'dont init'",
                path.display(),
                detail,
            ),
            Self::SchemaMismatch { found, expected } => {
                write!(f, "unsupported schema_version {found}; expected {expected}")
            }
            Self::CurieConflict { curie, existing_id } => {
                write!(f, "CURIE {curie} is already defined by {existing_id}")
            }
            Self::AmbiguousPrefix { prefix, candidates } => {
                write!(
                    f,
                    "prefix {prefix:?} matches {} entities: {}",
                    candidates.len(),
                    candidates.join(", ")
                )
            }
            Self::Malformed(message) => f.write_str(message),
            Self::DuplicateClaim {
                text_hash,
                existing_id,
            } => write!(
                f,
                "claim with text hash {text_hash} already exists as {existing_id}"
            ),
        }
    }
}

/// Resolved entity from [`Store::resolve_entity`].
#[derive(Debug, Clone)]
pub enum EntityResolution {
    Claim(ClaimRecord),
    Term(TermRecord),
}

impl std::error::Error for StoreError {}

impl Store {
    /// Open (or create) a store rooted at `project_root`. The DB lives at
    /// `project_root/.dont/db.cozo`.
    pub fn open_project(project_root: impl AsRef<Path>) -> Result<Self, StoreError> {
        let dont_dir = project_root.as_ref().join(".dont");
        Self::open_dont_dir(dont_dir)
    }

    /// Open (or create) a store whose DB lives directly inside `dont_dir`
    /// (`dont_dir/db.cozo`). Use this when `dont_dir` is already the `.dont/`
    /// directory (e.g. when `DONT_DIR` is set for test isolation).
    pub fn open_dont_dir(dont_dir: impl AsRef<Path>) -> Result<Self, StoreError> {
        let dont_dir = dont_dir.as_ref();
        #[cfg(unix)]
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(dont_dir)
            .map_err(StoreError::Io)?;
        #[cfg(not(unix))]
        std::fs::create_dir_all(dont_dir).map_err(StoreError::Io)?;
        let path = dont_dir.join("db.cozo");
        let lock_path = dont_dir.join("db.cozo.lock");
        let seq_path = dont_dir.join("tx.seq");

        // Hold the file lock across both DbInstance::new and ensure_schema.
        // DbInstance::new performs SQLite DDL (CREATE TABLE IF NOT EXISTS) which
        // is a write operation. Without the lock, concurrent processes can hit
        // SQLITE_BUSY (code 5) because CozoDB's SQLite backend sets no busy
        // timeout. The single file lock serialises the entire open+init sequence.
        Self::with_file_lock(&lock_path, || {
            // CozoDB's SQLite backend calls `.unwrap()` internally when it cannot
            // prepare its initial statement, which panics on a corrupt db file.
            // Catch the panic and convert it to a structured CorruptStore error so
            // callers get an actionable message that names the file.
            let db_path_clone = path.clone();
            let db_open_result =
                std::panic::catch_unwind(|| DbInstance::new("sqlite", &db_path_clone, ""));
            let db = match db_open_result {
                Ok(Ok(instance)) => instance,
                Ok(Err(err)) => {
                    return Err(StoreError::CorruptStore {
                        path: path.clone(),
                        detail: err.to_string(),
                    });
                }
                Err(_panic_payload) => {
                    return Err(StoreError::CorruptStore {
                        path: path.clone(),
                        detail: "storage engine panicked while opening the database (file may be corrupt or truncated)".to_string(),
                    });
                }
            };
            // The SQLite backend creates db.cozo using the OS default (subject to umask).
            // Tighten the permissions to 0o600 so the file is not world-readable.
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if path.exists() {
                    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
                        .map_err(StoreError::Io)?;
                }
            }
            let store = Self {
                db,
                path: path.clone(),
                lock_path: lock_path.clone(),
                seq_path: seq_path.clone(),
            };
            store.ensure_schema()?;
            Ok(store)
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn schema_version(&self) -> Result<i64, StoreError> {
        self.stored_schema_version()?
            .ok_or_else(|| StoreError::Malformed("missing schema_version metadata".to_string()))
    }

    #[doc(hidden)]
    pub fn set_schema_version_for_test(&self, version: i64) -> Result<(), StoreError> {
        self.with_write_lock(|store| {
            store.run(&format!(
                r#"?[key, value] <- [["schema_version", {}]] :put metadata {{key => value}}"#,
                version
            ))
        })
    }

    #[doc(hidden)]
    pub fn set_claim_hypotheses_for_test(
        &self,
        claim_id: &str,
        hypotheses: &[HypothesisRecord],
    ) -> Result<(), StoreError> {
        self.with_write_lock(|store| {
            let tx = store.next_tx()?;
            let value = serde_json::to_value(hypotheses).map_err(StoreError::from_err)?;
            store.put_datoms(&[Datom::assert(claim_id, "hypotheses", value, tx)])
        })
    }

    pub fn define_atom(
        &self,
        claim_id: &str,
        text: &str,
    ) -> Result<(AppendResult, usize), StoreError> {
        self.with_write_lock(|store| {
            let record = store
                .claim_by_id(claim_id)?
                .ok_or_else(|| StoreError::Malformed(format!("claim {claim_id} not found")))?;
            let tx = store.next_tx()?;
            let mut atoms = record.atoms;
            let idx = atoms.len();
            atoms.push(AtomRecord {
                idx,
                text: text.to_string(),
                status: "unverified".to_string(),
                evidence: vec![],
            });
            let value = serde_json::to_value(&atoms).map_err(StoreError::from_err)?;
            let event_id = prefixed_ulid("event");
            let now = now_rfc3339_seconds();
            store.put_datoms(&[
                Datom::assert(claim_id, "atoms", value, tx),
                Datom::assert(
                    &event_id,
                    "entity_type",
                    Value::String("event".to_string()),
                    tx,
                ),
                Datom::assert(
                    &event_id,
                    "claim_id",
                    Value::String(claim_id.to_string()),
                    tx,
                ),
                Datom::assert(
                    &event_id,
                    "kind",
                    Value::String("atom-defined".to_string()),
                    tx,
                ),
                Datom::assert(&event_id, "created_at", Value::String(now.clone()), tx),
            ])?;
            Ok((
                AppendResult {
                    id: claim_id.to_string(),
                    event_id,
                    tx,
                    created_at: now,
                },
                idx,
            ))
        })
    }

    pub fn dismiss_atom(
        &self,
        claim_id: &str,
        idx: usize,
        evidence: &[String],
    ) -> Result<AppendResult, StoreError> {
        self.with_write_lock(|store| {
            let record = store
                .claim_by_id(claim_id)?
                .ok_or_else(|| StoreError::Malformed(format!("claim {claim_id} not found")))?;
            let tx = store.next_tx()?;
            let mut atoms = record.atoms;
            let atom = atoms
                .get_mut(idx)
                .ok_or_else(|| StoreError::Malformed(format!("atom index {idx} out of range")))?;
            atom.status = "verified".to_string();
            atom.evidence.extend_from_slice(evidence);
            let value = serde_json::to_value(&atoms).map_err(StoreError::from_err)?;
            let event_id = prefixed_ulid("event");
            let now = now_rfc3339_seconds();
            store.put_datoms(&[
                Datom::assert(claim_id, "atoms", value, tx),
                Datom::assert(
                    &event_id,
                    "entity_type",
                    Value::String("event".to_string()),
                    tx,
                ),
                Datom::assert(
                    &event_id,
                    "claim_id",
                    Value::String(claim_id.to_string()),
                    tx,
                ),
                Datom::assert(
                    &event_id,
                    "kind",
                    Value::String("atom-dismissed".to_string()),
                    tx,
                ),
                Datom::assert(
                    &event_id,
                    "evidence",
                    serde_json::to_value(evidence).map_err(StoreError::from_err)?,
                    tx,
                ),
                Datom::assert(&event_id, "created_at", Value::String(now.clone()), tx),
            ])?;
            Ok(AppendResult {
                id: claim_id.to_string(),
                event_id,
                tx,
                created_at: now,
            })
        })
    }

    pub fn add_hypothesis(
        &self,
        claim_id: &str,
        text: &str,
    ) -> Result<(AppendResult, usize), StoreError> {
        self.with_write_lock(|store| {
            let record = store
                .claim_by_id(claim_id)?
                .ok_or_else(|| StoreError::Malformed(format!("claim {claim_id} not found")))?;
            let tx = store.next_tx()?;
            let mut hypotheses = record.hypotheses;
            let idx = hypotheses.len();
            hypotheses.push(HypothesisRecord {
                idx,
                text: text.to_string(),
                assessment: HypothesisAssessment {
                    supporting: vec![],
                    refuting: vec![],
                },
            });
            let value = serde_json::to_value(&hypotheses).map_err(StoreError::from_err)?;
            let event_id = prefixed_ulid("event");
            let now = now_rfc3339_seconds();
            store.put_datoms(&[
                Datom::assert(claim_id, "hypotheses", value, tx),
                Datom::assert(
                    &event_id,
                    "entity_type",
                    Value::String("event".to_string()),
                    tx,
                ),
                Datom::assert(
                    &event_id,
                    "claim_id",
                    Value::String(claim_id.to_string()),
                    tx,
                ),
                Datom::assert(
                    &event_id,
                    "kind",
                    Value::String("hypothesis-added".to_string()),
                    tx,
                ),
                Datom::assert(&event_id, "created_at", Value::String(now.clone()), tx),
            ])?;
            Ok((
                AppendResult {
                    id: claim_id.to_string(),
                    event_id,
                    tx,
                    created_at: now,
                },
                idx,
            ))
        })
    }

    pub fn assess_hypothesis(
        &self,
        claim_id: &str,
        idx: usize,
        supporting: &[String],
        refuting: &[String],
    ) -> Result<AppendResult, StoreError> {
        self.with_write_lock(|store| {
            let record = store
                .claim_by_id(claim_id)?
                .ok_or_else(|| StoreError::Malformed(format!("claim {claim_id} not found")))?;
            let tx = store.next_tx()?;
            let mut hypotheses = record.hypotheses;
            let h = hypotheses.get_mut(idx).ok_or_else(|| {
                StoreError::Malformed(format!("hypothesis index {idx} out of range"))
            })?;
            h.assessment.supporting.extend_from_slice(supporting);
            h.assessment.refuting.extend_from_slice(refuting);
            let value = serde_json::to_value(&hypotheses).map_err(StoreError::from_err)?;
            let event_id = prefixed_ulid("event");
            let now = now_rfc3339_seconds();
            store.put_datoms(&[
                Datom::assert(claim_id, "hypotheses", value, tx),
                Datom::assert(
                    &event_id,
                    "entity_type",
                    Value::String("event".to_string()),
                    tx,
                ),
                Datom::assert(
                    &event_id,
                    "claim_id",
                    Value::String(claim_id.to_string()),
                    tx,
                ),
                Datom::assert(
                    &event_id,
                    "kind",
                    Value::String("hypothesis-assessed".to_string()),
                    tx,
                ),
                Datom::assert(&event_id, "created_at", Value::String(now.clone()), tx),
            ])?;
            Ok(AppendResult {
                id: claim_id.to_string(),
                event_id,
                tx,
                created_at: now,
            })
        })
    }

    pub fn append_claim(
        &self,
        statement: &str,
        depends_on: &[String],
        confidence: Option<f64>,
    ) -> Result<AppendResult, StoreError> {
        let text_hash = claim_text_hash(statement);
        // Dedup check before acquiring the write lock (read-only query).
        if let Some(existing_id) = self.find_claim_by_text_hash(&text_hash)? {
            return Err(StoreError::DuplicateClaim {
                text_hash,
                existing_id,
            });
        }
        self.with_write_lock(|store| {
            let tx = store.next_tx()?;
            let claim_id = prefixed_ulid("claim");
            let event_id = prefixed_ulid("event");
            let now = now_rfc3339_seconds();
            let mut datoms = vec![
                Datom::assert(
                    &claim_id,
                    "entity_type",
                    Value::String("claim".to_string()),
                    tx,
                ),
                Datom::assert(
                    &claim_id,
                    "statement",
                    Value::String(statement.to_string()),
                    tx,
                ),
                Datom::assert(&claim_id, "text_hash", Value::String(text_hash.clone()), tx),
                Datom::assert(
                    &claim_id,
                    "status",
                    Value::String(Status::Unverified.as_str().to_string()),
                    tx,
                ),
                Datom::assert(&claim_id, "created_at", Value::String(now.clone()), tx),
                Datom::assert(
                    &event_id,
                    "entity_type",
                    Value::String("event".to_string()),
                    tx,
                ),
                Datom::assert(&event_id, "claim_id", Value::String(claim_id.clone()), tx),
                Datom::assert(
                    &event_id,
                    "kind",
                    Value::String(StoreEventKind::Concluded.as_str().to_string()),
                    tx,
                ),
                Datom::assert(&event_id, "created_at", Value::String(now.clone()), tx),
            ];
            if !depends_on.is_empty() {
                let arr = Value::Array(
                    depends_on
                        .iter()
                        .map(|c| Value::String(c.clone()))
                        .collect(),
                );
                datoms.push(Datom::assert(&claim_id, "depends_on", arr, tx));
            }
            if let Some(conf) = confidence {
                datoms.push(Datom::assert(
                    &claim_id,
                    "confidence",
                    serde_json::json!(conf),
                    tx,
                ));
            }
            store.put_datoms(&datoms)?;
            Ok(AppendResult {
                id: claim_id,
                event_id,
                tx,
                created_at: now,
            })
        })
    }

    pub fn append_term(
        &self,
        curie: &str,
        definition: &str,
        label: Option<&str>,
    ) -> Result<AppendResult, StoreError> {
        self.with_write_lock(|store| {
            if let Some(existing) = store.term_by_curie(curie)? {
                return Err(StoreError::CurieConflict {
                    curie: curie.to_string(),
                    existing_id: existing.id,
                });
            }
            let tx = store.next_tx()?;
            let term_id = prefixed_ulid("term");
            let event_id = prefixed_ulid("event");
            let now = now_rfc3339_seconds();
            let mut datoms = vec![
                Datom::assert(
                    &term_id,
                    "entity_type",
                    Value::String("term".to_string()),
                    tx,
                ),
                Datom::assert(&term_id, "curie", Value::String(curie.to_string()), tx),
                Datom::assert(
                    &term_id,
                    "definition",
                    Value::String(definition.to_string()),
                    tx,
                ),
                Datom::assert(
                    &term_id,
                    "status",
                    Value::String(Status::Unverified.as_str().to_string()),
                    tx,
                ),
                Datom::assert(&term_id, "created_at", Value::String(now.clone()), tx),
                Datom::assert(
                    &event_id,
                    "entity_type",
                    Value::String("event".to_string()),
                    tx,
                ),
                Datom::assert(&event_id, "entity_id", Value::String(term_id.clone()), tx),
                Datom::assert(
                    &event_id,
                    "kind",
                    Value::String(StoreEventKind::Defined.as_str().to_string()),
                    tx,
                ),
                Datom::assert(&event_id, "created_at", Value::String(now.clone()), tx),
            ];
            if let Some(lbl) = label {
                datoms.push(Datom::assert(
                    &term_id,
                    "label",
                    Value::String(lbl.to_string()),
                    tx,
                ));
            }
            store.put_datoms(&datoms)?;
            Ok(AppendResult {
                id: term_id,
                event_id,
                tx,
                created_at: now,
            })
        })
    }

    pub fn append_imported_term(
        &self,
        curie: &str,
        definition: &str,
        label: Option<&str>,
        source: &str,
    ) -> Result<AppendResult, StoreError> {
        self.with_write_lock(|store| {
            if let Some(existing) = store.imported_term_by_curie(curie)? {
                return Err(StoreError::CurieConflict {
                    curie: curie.to_string(),
                    existing_id: existing.id,
                });
            }
            let tx = store.next_tx()?;
            let imported_term_id = prefixed_ulid("imported_term");
            let now = now_rfc3339_seconds();
            let mut datoms = vec![
                Datom::assert(
                    &imported_term_id,
                    "entity_type",
                    Value::String("imported_term".to_string()),
                    tx,
                ),
                Datom::assert(
                    &imported_term_id,
                    "curie",
                    Value::String(curie.to_string()),
                    tx,
                ),
                Datom::assert(
                    &imported_term_id,
                    "definition",
                    Value::String(definition.to_string()),
                    tx,
                ),
                Datom::assert(
                    &imported_term_id,
                    "source",
                    Value::String(source.to_string()),
                    tx,
                ),
                Datom::assert(
                    &imported_term_id,
                    "imported_at",
                    Value::String(now.clone()),
                    tx,
                ),
            ];
            if let Some(lbl) = label {
                datoms.push(Datom::assert(
                    &imported_term_id,
                    "label",
                    Value::String(lbl.to_string()),
                    tx,
                ));
            }
            store.put_datoms(&datoms)?;
            Ok(AppendResult {
                id: imported_term_id,
                event_id: String::new(),
                tx,
                created_at: now,
            })
        })
    }

    pub fn append_status_change(
        &self,
        claim_id: &str,
        from_status: Status,
        to_status: Status,
        event: StoreEvent,
    ) -> Result<AppendResult, StoreError> {
        self.with_write_lock(|store| {
            let tx = store.next_tx()?;
            let event_id = prefixed_ulid("event");
            let now = now_rfc3339_seconds();
            let mut datoms = vec![
                Datom::retract(
                    claim_id,
                    "status",
                    Value::String(from_status.as_str().to_string()),
                    tx,
                ),
                Datom::assert(
                    claim_id,
                    "status",
                    Value::String(to_status.as_str().to_string()),
                    tx,
                ),
                Datom::assert(
                    &event_id,
                    "entity_type",
                    Value::String("event".to_string()),
                    tx,
                ),
                Datom::assert(
                    &event_id,
                    "claim_id",
                    Value::String(claim_id.to_string()),
                    tx,
                ),
                Datom::assert(
                    &event_id,
                    "kind",
                    Value::String(event.kind.as_str().to_string()),
                    tx,
                ),
                Datom::assert(&event_id, "created_at", Value::String(now.clone()), tx),
            ];
            if let Some(note) = event.note {
                datoms.push(Datom::assert(&event_id, "note", Value::String(note), tx));
            }
            if !event.evidence.is_empty() {
                datoms.push(Datom::assert(
                    &event_id,
                    "evidence",
                    Value::Array(event.evidence.clone()),
                    tx,
                ));
            }
            store.put_datoms(&datoms)?;
            Ok(AppendResult {
                id: claim_id.to_string(),
                event_id,
                tx,
                created_at: now,
            })
        })
    }

    /// Status change for terms — links the event via `entity_id` instead of `claim_id`.
    pub fn append_term_status_change(
        &self,
        term_id: &str,
        from_status: Status,
        to_status: Status,
        event: StoreEvent,
    ) -> Result<AppendResult, StoreError> {
        self.with_write_lock(|store| {
            let tx = store.next_tx()?;
            let event_id = prefixed_ulid("event");
            let now = now_rfc3339_seconds();
            let mut datoms = vec![
                Datom::retract(
                    term_id,
                    "status",
                    Value::String(from_status.as_str().to_string()),
                    tx,
                ),
                Datom::assert(
                    term_id,
                    "status",
                    Value::String(to_status.as_str().to_string()),
                    tx,
                ),
                Datom::assert(
                    &event_id,
                    "entity_type",
                    Value::String("event".to_string()),
                    tx,
                ),
                Datom::assert(
                    &event_id,
                    "entity_id",
                    Value::String(term_id.to_string()),
                    tx,
                ),
                Datom::assert(
                    &event_id,
                    "kind",
                    Value::String(event.kind.as_str().to_string()),
                    tx,
                ),
                Datom::assert(&event_id, "created_at", Value::String(now.clone()), tx),
            ];
            if let Some(note) = event.note {
                datoms.push(Datom::assert(&event_id, "note", Value::String(note), tx));
            }
            if !event.evidence.is_empty() {
                datoms.push(Datom::assert(
                    &event_id,
                    "evidence",
                    Value::Array(event.evidence.clone()),
                    tx,
                ));
            }
            store.put_datoms(&datoms)?;
            Ok(AppendResult {
                id: term_id.to_string(),
                event_id,
                tx,
                created_at: now,
            })
        })
    }

    /// Append an evidence-only event without changing the claim's status.
    /// Used when dismissing an already-verified claim (Phase 8).
    pub fn append_evidence_event(
        &self,
        claim_id: &str,
        event: StoreEvent,
    ) -> Result<AppendResult, StoreError> {
        self.with_write_lock(|store| {
            let tx = store.next_tx()?;
            let event_id = prefixed_ulid("event");
            let now = now_rfc3339_seconds();
            let mut datoms = vec![
                Datom::assert(
                    &event_id,
                    "entity_type",
                    Value::String("event".to_string()),
                    tx,
                ),
                Datom::assert(
                    &event_id,
                    "claim_id",
                    Value::String(claim_id.to_string()),
                    tx,
                ),
                Datom::assert(
                    &event_id,
                    "kind",
                    Value::String(event.kind.as_str().to_string()),
                    tx,
                ),
                Datom::assert(&event_id, "created_at", Value::String(now.clone()), tx),
            ];
            if let Some(note) = event.note {
                datoms.push(Datom::assert(&event_id, "note", Value::String(note), tx));
            }
            if !event.evidence.is_empty() {
                datoms.push(Datom::assert(
                    &event_id,
                    "evidence",
                    Value::Array(event.evidence.clone()),
                    tx,
                ));
            }
            store.put_datoms(&datoms)?;
            Ok(AppendResult {
                id: claim_id.to_string(),
                event_id,
                tx,
                created_at: now,
            })
        })
    }

    /// Return all claims, each with its current state. Order is undefined; callers sort.
    ///
    /// Uses two batch queries (all claim datoms + all event datoms) to avoid
    /// N-per-claim round trips, keeping list latency sub-linear in claim count.
    pub fn list_claims(&self) -> Result<Vec<ClaimRecord>, StoreError> {
        // 1. All datoms for all claim entities in one query
        let claim_rows = self.query_rows(
            r#"?[entity, attribute, value, tx, assert_bit] :=
                *datoms[entity, "entity_type", "claim", _, true],
                *datoms[entity, attribute, value, tx, assert_bit]"#,
        )?;
        let claim_datoms: Vec<Datom> = claim_rows
            .into_iter()
            .map(row_to_datom)
            .collect::<Result<_, _>>()?;

        // 2. All event datoms for all claim-owned events in one query
        let event_rows = self.query_rows(
            r#"?[ev_entity, attribute, value, tx, assert_bit] :=
                *datoms[_claim, "entity_type", "claim", _, true],
                *datoms[ev_entity, "claim_id", _claim, _, true],
                *datoms[ev_entity, attribute, value, tx, assert_bit]"#,
        )?;
        let event_datoms: Vec<Datom> = event_rows
            .into_iter()
            .map(row_to_datom)
            .collect::<Result<_, _>>()?;

        // Group event datoms by event entity
        let mut events_by_ev: std::collections::HashMap<String, Vec<&Datom>> =
            std::collections::HashMap::new();
        for d in &event_datoms {
            events_by_ev.entry(d.entity.clone()).or_default().push(d);
        }

        // Resolve claim_id for each event entity so we can group by claim
        let mut events_by_claim: std::collections::HashMap<String, Vec<EventRecord>> =
            std::collections::HashMap::new();
        for (ev_id, datoms) in &events_by_ev {
            let claim_id = datoms
                .iter()
                .filter(|d| d.attribute == "claim_id" && d.assert_bit)
                .max_by_key(|d| d.tx)
                .and_then(|d| d.value.as_str())
                .map(str::to_string);
            if let Some(cid) = claim_id {
                let record = event_from_datoms(ev_id.clone(), datoms.iter().copied())?;
                events_by_claim.entry(cid).or_default().push(record);
            }
        }

        // Group claim datoms by entity
        let mut claim_datoms_by_id: std::collections::HashMap<String, Vec<&Datom>> =
            std::collections::HashMap::new();
        for d in &claim_datoms {
            claim_datoms_by_id
                .entry(d.entity.clone())
                .or_default()
                .push(d);
        }

        // Build ClaimRecord for each entity
        let mut records = Vec::new();
        for (id, datoms) in claim_datoms_by_id {
            let statement = datoms
                .iter()
                .filter(|d| d.attribute == "statement" && d.assert_bit)
                .max_by_key(|d| d.tx)
                .and_then(|d| d.value.as_str())
                .ok_or_else(|| StoreError::Malformed(format!("claim {id} has no statement")))?
                .to_string();
            let status_str = datoms
                .iter()
                .filter(|d| d.attribute == "status" && d.assert_bit)
                .max_by_key(|d| d.tx)
                .and_then(|d| d.value.as_str())
                .ok_or_else(|| StoreError::Malformed(format!("claim {id} has no status")))?;
            let status = parse_status(status_str)?;
            let created_at = datoms
                .iter()
                .filter(|d| d.attribute == "created_at" && d.assert_bit)
                .max_by_key(|d| d.tx)
                .and_then(|d| d.value.as_str())
                .unwrap_or_default()
                .to_string();
            let mut events = events_by_claim.remove(&id).unwrap_or_default();
            events.sort_by_key(|e| e.tx);
            let depends_on = datoms
                .iter()
                .filter(|d| d.attribute == "depends_on" && d.assert_bit)
                .max_by_key(|d| d.tx)
                .and_then(|d| d.value.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(ToString::to_string))
                        .collect()
                })
                .unwrap_or_default();
            let atoms = datoms
                .iter()
                .filter(|d| d.attribute == "atoms" && d.assert_bit)
                .max_by_key(|d| d.tx)
                .map(|d| atoms_from_value(&d.value))
                .transpose()?
                .unwrap_or_default();
            let hypotheses = datoms
                .iter()
                .filter(|d| d.attribute == "hypotheses" && d.assert_bit)
                .max_by_key(|d| d.tx)
                .map(|d| hypotheses_from_value(&d.value))
                .transpose()?
                .unwrap_or_default();
            let confidence = datoms
                .iter()
                .filter(|d| d.attribute == "confidence" && d.assert_bit)
                .max_by_key(|d| d.tx)
                .and_then(|d| d.value.as_f64());
            records.push(ClaimRecord {
                id,
                statement,
                status,
                depends_on,
                atoms,
                hypotheses,
                created_at,
                events,
                confidence,
            });
        }
        Ok(records)
    }

    /// Return all terms, each with its current state. Uses two batch queries to avoid
    /// N-per-term round trips, matching the approach used by `list_claims`.
    pub fn list_terms(&self) -> Result<Vec<TermRecord>, StoreError> {
        // 1. All datoms for all term entities in one query
        let term_rows = self.query_rows(
            r#"?[entity, attribute, value, tx, assert_bit] :=
                *datoms[entity, "entity_type", "term", _, true],
                *datoms[entity, attribute, value, tx, assert_bit]"#,
        )?;
        let term_datoms: Vec<Datom> = term_rows
            .into_iter()
            .map(row_to_datom)
            .collect::<Result<_, _>>()?;

        // 2. All event datoms for all term-owned events (linked via entity_id)
        let event_rows = self.query_rows(
            r#"?[ev_entity, attribute, value, tx, assert_bit] :=
                *datoms[_term, "entity_type", "term", _, true],
                *datoms[ev_entity, "entity_id", _term, _, true],
                *datoms[ev_entity, attribute, value, tx, assert_bit]"#,
        )?;
        let event_datoms: Vec<Datom> = event_rows
            .into_iter()
            .map(row_to_datom)
            .collect::<Result<_, _>>()?;

        // Group event datoms by event entity
        let mut events_by_ev: std::collections::HashMap<String, Vec<&Datom>> =
            std::collections::HashMap::new();
        for d in &event_datoms {
            events_by_ev.entry(d.entity.clone()).or_default().push(d);
        }

        // Resolve entity_id for each event so we can group by term
        let mut events_by_term: std::collections::HashMap<String, Vec<EventRecord>> =
            std::collections::HashMap::new();
        for (ev_id, datoms) in &events_by_ev {
            let term_id = datoms
                .iter()
                .filter(|d| d.attribute == "entity_id" && d.assert_bit)
                .max_by_key(|d| d.tx)
                .and_then(|d| d.value.as_str())
                .map(str::to_string);
            if let Some(tid) = term_id {
                let record = event_from_datoms(ev_id.clone(), datoms.iter().copied())?;
                events_by_term.entry(tid).or_default().push(record);
            }
        }

        // Group term datoms by entity
        let mut term_datoms_by_id: std::collections::HashMap<String, Vec<&Datom>> =
            std::collections::HashMap::new();
        for d in &term_datoms {
            term_datoms_by_id
                .entry(d.entity.clone())
                .or_default()
                .push(d);
        }

        // Build TermRecord for each entity
        let mut records = Vec::new();
        for (id, datoms) in term_datoms_by_id {
            let curie = datoms
                .iter()
                .filter(|d| d.attribute == "curie" && d.assert_bit)
                .max_by_key(|d| d.tx)
                .and_then(|d| d.value.as_str())
                .ok_or_else(|| StoreError::Malformed(format!("term {id} has no curie")))?
                .to_string();
            let definition = datoms
                .iter()
                .filter(|d| d.attribute == "definition" && d.assert_bit)
                .max_by_key(|d| d.tx)
                .and_then(|d| d.value.as_str())
                .ok_or_else(|| StoreError::Malformed(format!("term {id} has no definition")))?
                .to_string();
            let status_str = datoms
                .iter()
                .filter(|d| d.attribute == "status" && d.assert_bit)
                .max_by_key(|d| d.tx)
                .and_then(|d| d.value.as_str())
                .ok_or_else(|| StoreError::Malformed(format!("term {id} has no status")))?;
            let status = parse_status(status_str)?;
            let created_at = datoms
                .iter()
                .filter(|d| d.attribute == "created_at" && d.assert_bit)
                .max_by_key(|d| d.tx)
                .and_then(|d| d.value.as_str())
                .unwrap_or_default()
                .to_string();
            let label = datoms
                .iter()
                .filter(|d| d.attribute == "label" && d.assert_bit)
                .max_by_key(|d| d.tx)
                .and_then(|d| d.value.as_str())
                .map(ToString::to_string);
            let mut events = events_by_term.remove(&id).unwrap_or_default();
            events.sort_by_key(|event| event.tx);
            records.push(TermRecord {
                id,
                curie,
                label,
                definition,
                status,
                created_at,
                events,
            });
        }
        Ok(records)
    }

    pub fn term_curie_exists(&self, curie: &str) -> Result<bool, StoreError> {
        let script = format!(
            r#"?[entity] := *datoms[entity, "curie", {}, _, true], *datoms[entity, "entity_type", "term", _, true]"#,
            json_string(curie)
        );
        let rows = self.query_rows(&script)?;
        Ok(!rows.is_empty())
    }

    pub fn claim_by_id(&self, claim_id: &str) -> Result<Option<ClaimRecord>, StoreError> {
        let datoms = self.datoms_for_entity(claim_id)?;
        if datoms.is_empty() {
            return Ok(None);
        }
        let statement = latest_asserted_value(&datoms, "statement")
            .and_then(Value::as_str)
            .ok_or_else(|| StoreError::Malformed(format!("claim {claim_id} has no statement")))?
            .to_string();
        let status = latest_asserted_value(&datoms, "status")
            .and_then(Value::as_str)
            .ok_or_else(|| StoreError::Malformed(format!("claim {claim_id} has no status")))
            .and_then(parse_status)?;
        let created_at = latest_asserted_value(&datoms, "created_at")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let depends_on = latest_asserted_value(&datoms, "depends_on")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(ToString::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let atoms = latest_asserted_value(&datoms, "atoms")
            .map(atoms_from_value)
            .transpose()?
            .unwrap_or_default();
        let hypotheses = latest_asserted_value(&datoms, "hypotheses")
            .map(hypotheses_from_value)
            .transpose()?
            .unwrap_or_default();
        let confidence = latest_asserted_value(&datoms, "confidence").and_then(Value::as_f64);
        let mut events = self.events_for_claim(claim_id)?;
        events.sort_by_key(|event| event.tx);
        Ok(Some(ClaimRecord {
            id: claim_id.to_string(),
            statement,
            status,
            depends_on,
            atoms,
            hypotheses,
            created_at,
            events,
            confidence,
        }))
    }

    pub fn term_by_id(&self, term_id: &str) -> Result<Option<TermRecord>, StoreError> {
        let datoms = self.datoms_for_entity(term_id)?;
        if datoms.is_empty() {
            return Ok(None);
        }
        let curie = latest_asserted_value(&datoms, "curie")
            .and_then(Value::as_str)
            .ok_or_else(|| StoreError::Malformed(format!("term {term_id} has no curie")))?
            .to_string();
        let definition = latest_asserted_value(&datoms, "definition")
            .and_then(Value::as_str)
            .ok_or_else(|| StoreError::Malformed(format!("term {term_id} has no definition")))?
            .to_string();
        let status = latest_asserted_value(&datoms, "status")
            .and_then(Value::as_str)
            .ok_or_else(|| StoreError::Malformed(format!("term {term_id} has no status")))
            .and_then(parse_status)?;
        let created_at = latest_asserted_value(&datoms, "created_at")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let label = latest_asserted_value(&datoms, "label")
            .and_then(Value::as_str)
            .map(ToString::to_string);
        let mut events = self.events_for_entity(term_id)?;
        events.sort_by_key(|event| event.tx);
        Ok(Some(TermRecord {
            id: term_id.to_string(),
            curie,
            label,
            definition,
            status,
            created_at,
            events,
        }))
    }

    pub fn term_by_curie(&self, curie: &str) -> Result<Option<TermRecord>, StoreError> {
        let script = format!(
            r#"?[entity] := *datoms[entity, "curie", {}, _, true], *datoms[entity, "entity_type", "term", _, true]"#,
            json_string(curie)
        );
        let rows = self.query_rows(&script)?;
        let Some(term_id) = rows
            .first()
            .and_then(|row| row.first())
            .and_then(Value::as_str)
        else {
            return Ok(None);
        };
        self.term_by_id(term_id)
    }

    pub fn imported_term_by_curie(
        &self,
        curie: &str,
    ) -> Result<Option<ImportedTermRecord>, StoreError> {
        let script = format!(
            r#"?[entity] := *datoms[entity, "curie", {}, _, true], *datoms[entity, "entity_type", "imported_term", _, true]"#,
            json_string(curie)
        );
        let rows = self.query_rows(&script)?;
        let Some(imported_term_id) = rows
            .first()
            .and_then(|row| row.first())
            .and_then(Value::as_str)
        else {
            return Ok(None);
        };
        let datoms = self.datoms_for_entity(imported_term_id)?;
        let definition = latest_asserted_value(&datoms, "definition")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                StoreError::Malformed(format!(
                    "imported_term {imported_term_id} has no definition"
                ))
            })?
            .to_string();
        let source = latest_asserted_value(&datoms, "source")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                StoreError::Malformed(format!("imported_term {imported_term_id} has no source"))
            })?
            .to_string();
        let imported_at = latest_asserted_value(&datoms, "imported_at")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let label = latest_asserted_value(&datoms, "label")
            .and_then(Value::as_str)
            .map(ToString::to_string);
        Ok(Some(ImportedTermRecord {
            id: imported_term_id.to_string(),
            curie: curie.to_string(),
            label,
            definition,
            source,
            imported_at,
        }))
    }

    pub fn resolve_curie_reference(
        &self,
        curie: &str,
    ) -> Result<Option<CurieResolution>, StoreError> {
        if let Some(term) = self.term_by_curie(curie)? {
            return Ok(Some(CurieResolution::Coined(term)));
        }
        self.imported_term_by_curie(curie)
            .map(|opt| opt.map(CurieResolution::Imported))
    }

    /// Resolve an entity identifier, CURIE, or short ULID prefix to a concrete record.
    ///
    /// Resolution rules (in priority order):
    /// 1. `claim:SUFFIX` — exact lookup if suffix is 26 chars; prefix search otherwise.
    /// 2. `term:SUFFIX`  — same as above for terms.
    /// 3. `NS:local` (any other colon form) — CURIE lookup.
    /// 4. Bare string (no colon) — prefix search across both claims and terms.
    ///
    /// Returns `Ok(None)` when no entity matches. Returns `Err(AmbiguousPrefix)` when
    /// more than one entity matches a prefix.
    pub fn resolve_entity(&self, input: &str) -> Result<Option<EntityResolution>, StoreError> {
        const ULID_LEN: usize = 26;

        if input.trim().is_empty() {
            return Ok(None);
        }

        if let Some(suffix) = input.strip_prefix("claim:") {
            if suffix.len() == ULID_LEN {
                return self
                    .claim_by_id(input)
                    .map(|opt| opt.map(EntityResolution::Claim));
            }
            let full_prefix = input; // "claim:<partial>"
            let candidates = self.ids_by_entity_type_and_prefix("claim", full_prefix)?;
            return self.resolve_candidates(input, candidates, |id| {
                self.claim_by_id(id).map(|o| o.map(EntityResolution::Claim))
            });
        }

        if let Some(suffix) = input.strip_prefix("term:") {
            if suffix.len() == ULID_LEN {
                return self
                    .term_by_id(input)
                    .map(|opt| opt.map(EntityResolution::Term));
            }
            let full_prefix = input;
            let candidates = self.ids_by_entity_type_and_prefix("term", full_prefix)?;
            return self.resolve_candidates(input, candidates, |id| {
                self.term_by_id(id).map(|o| o.map(EntityResolution::Term))
            });
        }

        if input.contains(':') {
            // CURIE
            return self
                .term_by_curie(input)
                .map(|opt| opt.map(EntityResolution::Term));
        }

        // Bare prefix — search both namespaces
        let claim_prefix = format!("claim:{input}");
        let term_prefix = format!("term:{input}");
        let mut candidates: Vec<(String, &str)> = vec![];
        for id in self.ids_by_entity_type_and_prefix("claim", &claim_prefix)? {
            candidates.push((id, "claim"));
        }
        for id in self.ids_by_entity_type_and_prefix("term", &term_prefix)? {
            candidates.push((id, "term"));
        }
        if candidates.len() > 1 {
            return Err(StoreError::AmbiguousPrefix {
                prefix: input.to_string(),
                candidates: candidates.into_iter().map(|(id, _)| id).collect(),
            });
        }
        match candidates.into_iter().next() {
            None => Ok(None),
            Some((id, "claim")) => self
                .claim_by_id(&id)
                .map(|o| o.map(EntityResolution::Claim)),
            Some((id, _)) => self.term_by_id(&id).map(|o| o.map(EntityResolution::Term)),
        }
    }

    // full_prefix must be in "type:suffix" form — IDs are stored with the type prefix.
    // Comparison is case-insensitive to match Crockford base32 ULIDs.
    fn ids_by_entity_type_and_prefix(
        &self,
        entity_type: &str,
        full_prefix: &str,
    ) -> Result<Vec<String>, StoreError> {
        let prefix_upper = full_prefix.to_uppercase();
        let script = format!(
            r#"?[entity] := *datoms[entity, "entity_type", {}, _, true]"#,
            json_string(entity_type)
        );
        let rows = self.query_rows(&script)?;
        Ok(rows
            .into_iter()
            .filter_map(|row| {
                row.into_iter()
                    .next()
                    .and_then(|v| v.as_str().map(str::to_string))
            })
            .filter(|id| id.to_uppercase().starts_with(&prefix_upper))
            .collect())
    }

    fn resolve_candidates<F>(
        &self,
        prefix: &str,
        candidates: Vec<String>,
        fetch: F,
    ) -> Result<Option<EntityResolution>, StoreError>
    where
        F: FnOnce(&str) -> Result<Option<EntityResolution>, StoreError>,
    {
        match candidates.len() {
            0 => Ok(None),
            1 => fetch(&candidates[0]),
            _ => Err(StoreError::AmbiguousPrefix {
                prefix: prefix.to_string(),
                candidates,
            }),
        }
    }

    pub fn datoms_for_entity(&self, entity: &str) -> Result<Vec<Datom>, StoreError> {
        let script = format!(
            r#"?[entity, attribute, value, tx, assert_bit] := *datoms[{}, attribute, value, tx, assert_bit], entity = {}"#,
            json_string(entity),
            json_string(entity)
        );
        let rows = self.query_rows(&script)?;
        rows.into_iter().map(row_to_datom).collect()
    }

    fn ensure_schema(&self) -> Result<(), StoreError> {
        self.run(r#"%ignore_error { :create datoms {entity: String, attribute: String, value: Any, tx: Int => assert_bit: Bool} }"#)?;
        self.run(r#"%ignore_error { :create metadata {key: String => value: Any} }"#)?;
        match self.stored_schema_version()? {
            Some(SCHEMA_VERSION) => Ok(()),
            Some(found) => Err(StoreError::SchemaMismatch {
                found,
                expected: SCHEMA_VERSION,
            }),
            None => self.run(&format!(
                r#"?[key, value] <- [["schema_version", {}]] :put metadata {{key => value}}"#,
                SCHEMA_VERSION
            )),
        }
    }

    fn stored_schema_version(&self) -> Result<Option<i64>, StoreError> {
        let rows = self.query_rows(r#"?[value] := *metadata["schema_version", value]"#)?;
        Ok(rows
            .first()
            .and_then(|row| row.first())
            .and_then(Value::as_i64))
    }

    fn next_tx(&self) -> Result<i64, StoreError> {
        // Read the last-used tx from the counter file, or seed it from the DB on first use.
        // Called only inside with_write_lock, so the file lock serialises this across processes.
        let last = if self.seq_path.exists() {
            std::fs::read_to_string(&self.seq_path)
                .map_err(StoreError::Io)?
                .trim()
                .parse::<i64>()
                .map_err(|_| StoreError::CorruptStore {
                    path: self.seq_path.clone(),
                    detail: "file does not contain a valid integer transaction counter".to_string(),
                })?
        } else {
            let rows =
                self.query_rows("?[max(tx)] := *datoms[entity, attribute, value, tx, assert_bit]")?;
            rows.first()
                .and_then(|row| row.first())
                .and_then(Value::as_i64)
                .unwrap_or(0)
        };
        let next = last + 1;
        write_restricted(&self.seq_path, next.to_string().as_bytes()).map_err(StoreError::Io)?;
        Ok(next)
    }

    fn events_for_claim(&self, claim_id: &str) -> Result<Vec<EventRecord>, StoreError> {
        let script = format!(
            r#"?[entity, attribute, value, tx, assert_bit] := *datoms[entity, "claim_id", {}, tx, true], *datoms[entity, attribute, value, tx, assert_bit]"#,
            json_string(claim_id)
        );
        self.events_from_query(&script)
    }

    fn events_for_entity(&self, entity_id: &str) -> Result<Vec<EventRecord>, StoreError> {
        let script = format!(
            r#"?[entity, attribute, value, tx, assert_bit] := *datoms[entity, "entity_id", {}, tx, true], *datoms[entity, attribute, value, tx, assert_bit]"#,
            json_string(entity_id)
        );
        self.events_from_query(&script)
    }

    fn events_from_query(&self, script: &str) -> Result<Vec<EventRecord>, StoreError> {
        let rows = self.query_rows(script)?;
        let datoms: Vec<Datom> = rows
            .into_iter()
            .map(row_to_datom)
            .collect::<Result<_, _>>()?;
        let mut event_ids: Vec<String> = datoms.iter().map(|d| d.entity.clone()).collect();
        event_ids.sort();
        event_ids.dedup();
        event_ids
            .into_iter()
            .map(|id| event_from_datoms(id.clone(), datoms.iter().filter(move |d| d.entity == id)))
            .collect()
    }

    fn put_datoms(&self, datoms: &[Datom]) -> Result<(), StoreError> {
        let rows = datoms
            .iter()
            .map(|d| {
                format!(
                    "[{}, {}, {}, {}, {}]",
                    json_string(&d.entity),
                    json_string(&d.attribute),
                    d.value,
                    d.tx,
                    d.assert_bit
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        self.run(&format!(
            "?[entity, attribute, value, tx, assert_bit] <- [{}] :put datoms {{entity, attribute, value, tx => assert_bit}}",
            rows
        ))
    }

    fn with_write_lock<T>(
        &self,
        f: impl FnOnce(&Self) -> Result<T, StoreError>,
    ) -> Result<T, StoreError> {
        Self::with_file_lock(&self.lock_path, || f(self))
    }

    // Acquire an exclusive advisory lock on `lock_path`, run `f`, then release
    // the lock. Used both for write serialisation (via `with_write_lock`) and
    // for the store-open sequence, so that `DbInstance::new` (which performs
    // SQLite DDL) is also protected and cannot cause SQLITE_BUSY (code 5) when
    // multiple processes initialise concurrently.
    fn with_file_lock<T>(
        lock_path: &Path,
        f: impl FnOnce() -> Result<T, StoreError>,
    ) -> Result<T, StoreError> {
        let lock = Self::open_lock_file_at(lock_path)?;
        lock.lock_exclusive().map_err(StoreError::Io)?;
        let result = f();
        let unlock_result = lock.unlock().map_err(StoreError::Io);
        match (result, unlock_result) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(error),
        }
    }

    fn open_lock_file_at(lock_path: &Path) -> Result<File, StoreError> {
        #[cfg(unix)]
        let f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .mode(0o600)
            .open(lock_path);
        #[cfg(not(unix))]
        let f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(lock_path);
        f.map_err(StoreError::Io)
    }

    /// Execute a rule violation query against the store.
    ///
    /// Rule scripts MUST return two columns: `[entity_id, detail]`.
    /// Returns `StoreError::Cozo` if the script fails to parse or execute.
    pub fn run_rule_query(&self, script: &str) -> Result<Vec<Vec<Value>>, StoreError> {
        self.query_rows(script)
    }

    fn run(&self, script: &str) -> Result<(), StoreError> {
        let result = self.db.run_script_str(script, "", false);
        let value: Value = serde_json::from_str(&result).map_err(StoreError::from_err)?;
        if value.get("ok").and_then(Value::as_bool) == Some(false) {
            return Err(StoreError::Cozo(value.to_string()));
        }
        Ok(())
    }

    fn query_rows(&self, script: &str) -> Result<Vec<Vec<Value>>, StoreError> {
        let result = self.db.run_script_str(script, "", true);
        let value: Value = serde_json::from_str(&result).map_err(StoreError::from_err)?;
        if value.get("ok").and_then(Value::as_bool) == Some(false) {
            return Err(StoreError::Cozo(value.to_string()));
        }
        serde_json::from_value(value.get("rows").cloned().unwrap_or(Value::Array(vec![])))
            .map_err(StoreError::from_err)
    }

    /// Return all events ordered by `created_at`, optionally filtered to [since, until).
    ///
    /// Both bounds are RFC 3339 strings. RFC 3339 sorts lexicographically when
    /// produced by the same formatter (seconds precision, UTC).
    pub fn all_events_in_scope(
        &self,
        since: Option<&str>,
        until: Option<&str>,
    ) -> Result<Vec<EventRecord>, StoreError> {
        let rows = self.query_rows(
            r#"?[ev_entity, attribute, value, tx, assert_bit] :=
                *datoms[ev_entity, "entity_type", "event", _, true],
                *datoms[ev_entity, attribute, value, tx, assert_bit]"#,
        )?;
        let datoms: Vec<Datom> = rows
            .into_iter()
            .map(row_to_datom)
            .collect::<Result<_, _>>()?;
        let mut ev_ids: Vec<String> = datoms.iter().map(|d| d.entity.clone()).collect();
        ev_ids.sort();
        ev_ids.dedup();
        let mut records: Vec<EventRecord> = ev_ids
            .into_iter()
            .map(|id| event_from_datoms(id.clone(), datoms.iter().filter(move |d| d.entity == id)))
            .collect::<Result<_, _>>()?;
        if let Some(s) = since {
            records.retain(|e| e.created_at.as_str() >= s);
        }
        if let Some(u) = until {
            records.retain(|e| e.created_at.as_str() < u);
        }
        records.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(records)
    }

    /// Count claims by their current status string (correctly resolved via list_claims).
    pub fn claim_counts_by_status(
        &self,
    ) -> Result<std::collections::HashMap<String, u64>, StoreError> {
        let claims = self.list_claims()?;
        let mut counts: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        for claim in claims {
            *counts.entry(claim.status.as_str().to_string()).or_insert(0) += 1;
        }
        Ok(counts)
    }

    /// Check dedup without writing. Returns Err(DuplicateClaim) if duplicate found.
    pub fn check_claim_dedup(&self, statement: &str) -> Result<(), StoreError> {
        let hash = claim_text_hash(statement);
        if let Some(existing_id) = self.find_claim_by_text_hash(&hash)? {
            return Err(StoreError::DuplicateClaim {
                text_hash: hash,
                existing_id,
            });
        }
        Ok(())
    }

    /// Find the ID of any existing claim whose text hash matches `hash`.
    pub fn find_claim_by_text_hash(&self, hash: &str) -> Result<Option<String>, StoreError> {
        let rows = self.query_rows(&format!(
            r#"?[entity] :=
                *datoms[entity, "entity_type", "claim", _, true],
                *datoms[entity, "text_hash", {}, _, true]"#,
            json_string(hash)
        ))?;
        Ok(rows
            .into_iter()
            .next()
            .and_then(|r| r.into_iter().next())
            .and_then(|v| {
                if let Value::String(s) = v {
                    Some(s)
                } else {
                    None
                }
            }))
    }

    /// Richer query for trust/flag events that includes the claim_id.
    pub fn trust_flag_events_with_claim_id(
        &self,
        since: Option<&str>,
        until: Option<&str>,
    ) -> Result<Vec<TrustEventRow>, StoreError> {
        // Query without requiring note (note is optional on flag events).
        let rows = self.query_rows(
            r#"?[ev_id, claim_id, kind, created_at] :=
                *datoms[ev_id, "entity_type", "event", _, true],
                *datoms[ev_id, "claim_id", claim_id, _, true],
                *datoms[ev_id, "kind", kind, _, true],
                *datoms[ev_id, "created_at", created_at, _, true],
                (kind == "trusted" || kind == "flagged")"#,
        )?;
        // Separately gather notes for events that have them.
        let note_rows = self.query_rows(
            r#"?[ev_id, note] :=
                *datoms[ev_id, "entity_type", "event", _, true],
                *datoms[ev_id, "note", note, _, true],
                *datoms[ev_id, "kind", kind, _, true],
                (kind == "trusted" || kind == "flagged")"#,
        )?;
        let mut notes: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        for row in note_rows {
            if let (Some(Value::String(ev_id)), Some(Value::String(note))) =
                (row.first(), row.get(1))
            {
                notes.insert(ev_id.clone(), note.clone());
            }
        }
        let mut result = Vec::new();
        for row in rows {
            let ev_id = row
                .first()
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let claim_id = row
                .get(1)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let kind = row
                .get(2)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let created_at = row
                .get(3)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if let (Some(s), false) = (since, created_at.is_empty())
                && created_at.as_str() < s
            {
                continue;
            }
            if let (Some(u), false) = (until, created_at.is_empty())
                && created_at.as_str() >= u
            {
                continue;
            }
            let note = notes.get(&ev_id).cloned();
            result.push(TrustEventRow {
                event_id: ev_id,
                claim_id,
                kind,
                note,
                created_at,
            });
        }
        Ok(result)
    }

    /// Count Trusted events in scope where the targeted claim is used as evidence
    /// (appears in depends_on) for another claim created before the doubt event.
    pub fn caught_contradiction_count(
        &self,
        since: Option<&str>,
        until: Option<&str>,
    ) -> Result<u64, StoreError> {
        let trust_rows = self.trust_flag_events_with_claim_id(since, until)?;
        let doubt_events: Vec<&TrustEventRow> =
            trust_rows.iter().filter(|r| r.kind == "trusted").collect();
        if doubt_events.is_empty() {
            return Ok(0);
        }
        // Build map: dep_id → list of (parent created_at) for claims that depend on it.
        let all_claims = self.list_claims()?;
        let mut dep_to_parents: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for claim in &all_claims {
            for dep_id in &claim.depends_on {
                dep_to_parents
                    .entry(dep_id.clone())
                    .or_default()
                    .push(claim.created_at.clone());
            }
        }
        let mut count = 0u64;
        for event in &doubt_events {
            if let Some(parents) = dep_to_parents.get(&event.claim_id) {
                // Use <= to handle same-second timestamps in tests/fast machines.
                if parents
                    .iter()
                    .any(|cat| cat.as_str() <= event.created_at.as_str())
                {
                    count += 1;
                }
            }
        }
        Ok(count)
    }
}

impl Datom {
    fn assert(entity: &str, attribute: &str, value: Value, tx: i64) -> Self {
        Self {
            entity: entity.to_string(),
            attribute: attribute.to_string(),
            value,
            tx,
            assert_bit: true,
        }
    }

    fn retract(entity: &str, attribute: &str, value: Value, tx: i64) -> Self {
        Self {
            entity: entity.to_string(),
            attribute: attribute.to_string(),
            value,
            tx,
            assert_bit: false,
        }
    }
}

impl StoreError {
    fn from_err(error: impl std::error::Error) -> Self {
        Self::Malformed(error.to_string())
    }
}

fn latest_asserted_value<'a>(datoms: &'a [Datom], attribute: &str) -> Option<&'a Value> {
    datoms
        .iter()
        .filter(|d| d.attribute == attribute && d.assert_bit)
        .max_by_key(|d| d.tx)
        .map(|d| &d.value)
}

fn event_from_datoms<'a>(
    id: String,
    datoms: impl Iterator<Item = &'a Datom>,
) -> Result<EventRecord, StoreError> {
    let datoms: Vec<&Datom> = datoms.collect();
    let kind = latest_asserted_ref(&datoms, "kind")
        .and_then(Value::as_str)
        .ok_or_else(|| StoreError::Malformed(format!("event {id} has no kind")))
        .and_then(StoreEventKind::from_str)?;
    let tx = datoms.iter().map(|d| d.tx).min().unwrap_or_default();
    let created_at = latest_asserted_ref(&datoms, "created_at")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let note = latest_asserted_ref(&datoms, "note")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let evidence: Vec<Value> = latest_asserted_ref(&datoms, "evidence")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    Ok(EventRecord {
        id,
        kind,
        note,
        evidence,
        tx,
        created_at,
    })
}

fn latest_asserted_ref<'a>(datoms: &[&'a Datom], attribute: &str) -> Option<&'a Value> {
    datoms
        .iter()
        .copied()
        .filter(|d| d.attribute == attribute && d.assert_bit)
        .max_by_key(|d| d.tx)
        .map(|d| &d.value)
}

fn row_to_datom(row: Vec<Value>) -> Result<Datom, StoreError> {
    let mut row = row.into_iter();
    Ok(Datom {
        entity: row
            .next()
            .and_then(|v| v.as_str().map(ToString::to_string))
            .ok_or_else(|| StoreError::Malformed("datom row missing entity".to_string()))?,
        attribute: row
            .next()
            .and_then(|v| v.as_str().map(ToString::to_string))
            .ok_or_else(|| StoreError::Malformed("datom row missing attribute".to_string()))?,
        value: row
            .next()
            .ok_or_else(|| StoreError::Malformed("datom row missing value".to_string()))?,
        tx: row
            .next()
            .and_then(|v| v.as_i64())
            .ok_or_else(|| StoreError::Malformed("datom row missing tx".to_string()))?,
        assert_bit: row
            .next()
            .and_then(|v| v.as_bool())
            .ok_or_else(|| StoreError::Malformed("datom row missing assert_bit".to_string()))?,
    })
}

fn prefixed_ulid(prefix: &str) -> String {
    format!("{prefix}:{}", Ulid::new())
}

fn now_rfc3339_seconds() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn atoms_from_value(value: &Value) -> Result<Vec<AtomRecord>, StoreError> {
    serde_json::from_value(value.clone()).map_err(StoreError::from_err)
}

fn hypotheses_from_value(value: &Value) -> Result<Vec<HypothesisRecord>, StoreError> {
    serde_json::from_value(value.clone()).map_err(StoreError::from_err)
}

/// Public wrapper around `now_rfc3339_seconds` for use in ephemeral responses.
pub fn now_rfc3339_pub() -> String {
    now_rfc3339_seconds()
}

/// Normalize claim text for dedup: lowercase + whitespace collapse.
///
/// Full NFC normalization is not applied here to avoid a heavyweight dependency;
/// the SHA-256 is computed over the ASCII-safe normalized form.
pub fn normalize_claim_text(text: &str) -> String {
    text.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Hex-encoded SHA-256 of the normalized claim text.
pub fn claim_text_hash(text: &str) -> String {
    let normalized = normalize_claim_text(text);
    let hash = Sha256::digest(normalized.as_bytes());
    format!("{hash:x}")
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("serializing string literal cannot fail")
}

#[cfg(test)]
mod data_model {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn duplicate_curie_insert_returns_curie_conflict_error() {
        let dir = TempDir::new().unwrap();
        let store = Store::open_dont_dir(dir.path()).unwrap();

        let first = store
            .append_term("WB:P001", "a first definition", None)
            .unwrap();
        let err = store
            .append_term("WB:P001", "a second definition", None)
            .unwrap_err();

        match err {
            StoreError::CurieConflict { curie, existing_id } => {
                assert_eq!(curie, "WB:P001");
                assert_eq!(existing_id, first.id);
            }
            other => panic!("expected CurieConflict, got {other:?}"),
        }

        let term = store.term_by_curie("WB:P001").unwrap().unwrap();
        assert_eq!(term.id, first.id);
        assert_eq!(term.definition, "a first definition");
    }

    #[test]
    fn imported_terms_resolve_without_entering_coined_term_table() {
        let dir = TempDir::new().unwrap();
        let store = Store::open_dont_dir(dir.path()).unwrap();

        store
            .append_imported_term(
                "EX:Observation",
                "imported definition",
                Some("Observation"),
                "linkml:basic",
            )
            .unwrap();

        assert!(store.term_by_curie("EX:Observation").unwrap().is_none());
        let resolved = store.resolve_curie_reference("EX:Observation").unwrap();
        assert!(matches!(resolved, Some(CurieResolution::Imported(_))));
    }
}
