use std::fmt;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use chrono::{SecondsFormat, Utc};
use cozo::DbInstance;
use fs2::FileExt;
use serde_json::Value;
use ulid::Ulid;

const SCHEMA_VERSION: i64 = 1;

pub struct Store {
    db: DbInstance,
    path: PathBuf,
    lock_path: PathBuf,
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
pub enum StoreStatus {
    Unverified,
    Verified,
    Doubted,
}

impl StoreStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Unverified => "unverified",
            Self::Verified => "verified",
            Self::Doubted => "doubted",
        }
    }

    fn from_str(value: &str) -> Result<Self, StoreError> {
        match value {
            "unverified" => Ok(Self::Unverified),
            "verified" => Ok(Self::Verified),
            "doubted" => Ok(Self::Doubted),
            _ => Err(StoreError::Malformed(format!("unknown status {value}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreEventKind {
    Concluded,
    Trusted,
    Dismissed,
}

impl StoreEventKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Concluded => "concluded",
            Self::Trusted => "trusted",
            Self::Dismissed => "dismissed",
        }
    }

    fn from_str(value: &str) -> Result<Self, StoreError> {
        match value {
            "concluded" => Ok(Self::Concluded),
            "trusted" => Ok(Self::Trusted),
            "dismissed" => Ok(Self::Dismissed),
            _ => Err(StoreError::Malformed(format!("unknown event kind {value}"))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreEvent {
    pub kind: StoreEventKind,
    pub note: Option<String>,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Datom {
    pub entity: String,
    pub attribute: String,
    pub value: Value,
    pub tx: i64,
    pub assert_bit: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClaimRecord {
    pub id: String,
    pub statement: String,
    pub status: StoreStatus,
    pub created_at: String,
    pub events: Vec<EventRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventRecord {
    pub id: String,
    pub kind: StoreEventKind,
    pub note: Option<String>,
    pub evidence: Vec<String>,
    pub tx: i64,
    pub created_at: String,
}

#[derive(Debug)]
pub enum StoreError {
    Io(std::io::Error),
    Cozo(String),
    SchemaMismatch { found: i64, expected: i64 },
    Malformed(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {error}"),
            Self::Cozo(message) => write!(f, "Cozo error: {message}"),
            Self::SchemaMismatch { found, expected } => {
                write!(f, "unsupported schema_version {found}; expected {expected}")
            }
            Self::Malformed(message) => f.write_str(message),
        }
    }
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
        std::fs::create_dir_all(dont_dir).map_err(StoreError::Io)?;
        let path = dont_dir.join("db.cozo");
        let lock_path = dont_dir.join("db.cozo.lock");
        let db = DbInstance::new("sqlite", &path, "")
            .map_err(|err| StoreError::Cozo(err.to_string()))?;
        let store = Self {
            db,
            path,
            lock_path,
        };
        store.with_write_lock(|store| store.ensure_schema())?;
        Ok(store)
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

    pub fn append_claim(&self, statement: &str) -> Result<AppendResult, StoreError> {
        self.with_write_lock(|store| {
            let tx = store.next_tx()?;
            let claim_id = prefixed_ulid("claim");
            let event_id = prefixed_ulid("event");
            let now = now_rfc3339_seconds();
            let datoms = vec![
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
                Datom::assert(
                    &claim_id,
                    "status",
                    Value::String(StoreStatus::Unverified.as_str().to_string()),
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
            store.put_datoms(&datoms)?;
            Ok(AppendResult {
                id: claim_id,
                event_id,
                tx,
                created_at: now,
            })
        })
    }

    pub fn append_status_change(
        &self,
        claim_id: &str,
        from_status: StoreStatus,
        to_status: StoreStatus,
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
                let arr = Value::Array(
                    event.evidence.iter().map(|u| Value::String(u.clone())).collect(),
                );
                datoms.push(Datom::assert(&event_id, "evidence", arr, tx));
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
                let arr = Value::Array(
                    event.evidence.iter().map(|u| Value::String(u.clone())).collect(),
                );
                datoms.push(Datom::assert(&event_id, "evidence", arr, tx));
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
    pub fn list_claims(&self) -> Result<Vec<ClaimRecord>, StoreError> {
        let rows = self.query_rows(
            r#"?[entity] := *datoms[entity, "entity_type", "claim", _, true]"#,
        )?;
        rows.into_iter()
            .map(|row| {
                let id = row
                    .into_iter()
                    .next()
                    .and_then(|v| v.as_str().map(str::to_string))
                    .ok_or_else(|| StoreError::Malformed("list_claims row missing entity".to_string()))?;
                self.claim_by_id(&id)?.ok_or_else(|| {
                    StoreError::Malformed(format!("claim {id} vanished during list"))
                })
            })
            .collect()
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
            .and_then(StoreStatus::from_str)?;
        let created_at = latest_asserted_value(&datoms, "created_at")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let mut events = self.events_for_claim(claim_id)?;
        events.sort_by_key(|event| event.tx);
        Ok(Some(ClaimRecord {
            id: claim_id.to_string(),
            statement,
            status,
            created_at,
            events,
        }))
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
        let rows =
            self.query_rows("?[max(tx)] := *datoms[entity, attribute, value, tx, assert_bit]")?;
        let max = rows
            .first()
            .and_then(|row| row.first())
            .and_then(Value::as_i64)
            .unwrap_or(0);
        Ok(max + 1)
    }

    fn events_for_claim(&self, claim_id: &str) -> Result<Vec<EventRecord>, StoreError> {
        let script = format!(
            r#"?[entity, attribute, value, tx, assert_bit] := *datoms[entity, "claim_id", {}, tx, true], *datoms[entity, attribute, value, tx, assert_bit]"#,
            json_string(claim_id)
        );
        let rows = self.query_rows(&script)?;
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
        let lock = self.open_lock_file()?;
        lock.lock_exclusive().map_err(StoreError::Io)?;
        let result = f(self);
        let unlock_result = lock.unlock().map_err(StoreError::Io);
        match (result, unlock_result) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(error),
        }
    }

    fn open_lock_file(&self) -> Result<File, StoreError> {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&self.lock_path)
            .map_err(StoreError::Io)
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
    let evidence: Vec<String> = latest_asserted_ref(&datoms, "evidence")
        .and_then(Value::as_array)
        .map(|arr| arr.iter().filter_map(|e| e.as_str().map(str::to_string)).collect())
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

fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("serializing string literal cannot fail")
}
