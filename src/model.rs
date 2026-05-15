use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Unverified,
    Verified,
    Doubted,
    Ignored,
    Locked,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unverified => "unverified",
            Self::Verified => "verified",
            Self::Doubted => "doubted",
            Self::Ignored => "ignored",
            Self::Locked => "locked",
        }
    }

    pub fn from_persisted_str(value: &str) -> Option<Self> {
        match value {
            "unverified" => Some(Self::Unverified),
            "verified" => Some(Self::Verified),
            "doubted" => Some(Self::Doubted),
            "ignored" => Some(Self::Ignored),
            "locked" => Some(Self::Locked),
            _ => None,
        }
    }
}

/// Tagged view of a user-supplied entity identifier.
///
/// Avoids stringly-typed `id.starts_with("term:")` dispatch at command sites.
/// The wrapped `&str` is the original literal id — `as_str()` round-trips it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityId<'a> {
    Claim(&'a str),
    Term(&'a str),
}

impl<'a> EntityId<'a> {
    const TERM_PREFIX: &'static str = "term:";

    pub fn parse(s: &'a str) -> Self {
        if s.starts_with(Self::TERM_PREFIX) {
            Self::Term(s)
        } else {
            Self::Claim(s)
        }
    }

    pub fn as_str(&self) -> &'a str {
        match self {
            Self::Claim(s) | Self::Term(s) => s,
        }
    }
}

impl std::fmt::Display for EntityId<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionError {
    pub code: String,
    pub message: String,
    pub from_status: Status,
    pub entity_id: Option<String>,
}

pub fn trust(from: Status) -> Result<Status, TransitionError> {
    match from {
        Status::Unverified | Status::Verified => Ok(Status::Doubted),
        Status::Doubted | Status::Ignored | Status::Locked => Err(TransitionError {
            code: "invalid-transition".to_string(),
            message: format!("cannot trust a {from:?} entity"),
            from_status: from,
            entity_id: None,
        }),
    }
}

pub fn reopen(from: Status) -> Result<Status, TransitionError> {
    match from {
        Status::Ignored => Ok(Status::Unverified),
        Status::Unverified | Status::Verified | Status::Doubted | Status::Locked => {
            Err(TransitionError {
                code: "invalid-transition".to_string(),
                message: format!("cannot reopen a {from:?} entity — only ignored entities can be reopened"),
                from_status: from,
                entity_id: None,
            })
        }
    }
}

pub fn ignore(from: Status) -> Result<Status, TransitionError> {
    match from {
        Status::Unverified | Status::Verified | Status::Doubted => Ok(Status::Ignored),
        Status::Ignored => Err(TransitionError {
            code: "invalid-transition".to_string(),
            message: "cannot ignore an already-ignored entity".to_string(),
            from_status: from,
            entity_id: None,
        }),
        Status::Locked => Err(TransitionError {
            code: "invalid-transition".to_string(),
            message: "cannot ignore a locked entity".to_string(),
            from_status: from,
            entity_id: None,
        }),
    }
}

pub fn flag(from: Status) -> Result<Status, TransitionError> {
    match from {
        Status::Unverified | Status::Doubted => Ok(Status::Verified),
        Status::Verified | Status::Ignored | Status::Locked => Err(TransitionError {
            code: "invalid-transition".to_string(),
            message: format!("cannot flag a {from:?} entity as a status transition"),
            from_status: from,
            entity_id: None,
        }),
    }
}

pub fn undoubt(from: Status) -> Result<Status, TransitionError> {
    match from {
        Status::Doubted => Ok(Status::Unverified),
        Status::Unverified | Status::Verified | Status::Ignored | Status::Locked => {
            Err(TransitionError {
                code: "invalid-transition".to_string(),
                message: format!(
                    "cannot undoubt a {from:?} entity — only doubted entities can be undoubted"
                ),
                from_status: from,
                entity_id: None,
            })
        }
    }
}

pub fn lock(from: Status) -> Result<Status, TransitionError> {
    match from {
        Status::Verified => Ok(Status::Locked),
        Status::Unverified | Status::Doubted | Status::Ignored | Status::Locked => {
            Err(TransitionError {
                code: "invalid-transition".to_string(),
                message: format!("cannot lock a {from:?} entity"),
                from_status: from,
                entity_id: None,
            })
        }
    }
}
