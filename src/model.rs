use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Unverified,
    Verified,
    Doubted,
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
        Status::Doubted => Err(TransitionError {
            code: "invalid-transition".to_string(),
            message: format!(
                "cannot trust a {from:?} entity — it is already doubted"
            ),
            from_status: from,
            entity_id: None,
        }),
    }
}

pub fn dismiss(from: Status) -> Result<Status, TransitionError> {
    match from {
        Status::Unverified | Status::Doubted => Ok(Status::Verified),
        Status::Verified => Err(TransitionError {
            code: "invalid-transition".to_string(),
            message: "cannot dismiss a verified entity as a status transition — \
                      use evidence append (Phase 8)"
                .to_string(),
            from_status: from,
            entity_id: None,
        }),
    }
}
