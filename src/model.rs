use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Unverified,
    Verified,
    Doubted,
    Ignored,
    Locked,
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

pub fn dismiss(from: Status) -> Result<Status, TransitionError> {
    match from {
        Status::Unverified | Status::Doubted => Ok(Status::Verified),
        Status::Verified | Status::Ignored | Status::Locked => Err(TransitionError {
            code: "invalid-transition".to_string(),
            message: format!("cannot dismiss a {from:?} entity as a status transition"),
            from_status: from,
            entity_id: None,
        }),
    }
}
