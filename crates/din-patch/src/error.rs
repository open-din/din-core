use thiserror::Error;

#[derive(Debug, Error)]
pub enum PatchError {
    #[error("failed to parse patch JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("patch document must be version {expected}, got {actual}")]
    UnsupportedVersion { expected: u32, actual: u32 },
    #[error("patch nodes must include a non-empty id")]
    MissingNodeId,
    #[error("patch node \"{node_id}\" has mismatched type metadata")]
    MismatchedNodeType { node_id: String },
    #[error("patch connection \"{connection_id}\" references missing source node \"{source_id}\"")]
    MissingSourceNode {
        connection_id: String,
        source_id: String,
    },
    #[error("patch connection \"{connection_id}\" references missing target node \"{target_id}\"")]
    MissingTargetNode {
        connection_id: String,
        target_id: String,
    },
    #[error("patch connection \"{connection_id}\" cannot connect a node to itself")]
    SelfConnection { connection_id: String },
    #[error("patch connection \"{connection_id}\" uses unsupported source handle \"{handle}\"")]
    UnsupportedSourceHandle {
        connection_id: String,
        handle: String,
    },
    #[error("patch connection \"{connection_id}\" uses unsupported target handle \"{handle}\"")]
    UnsupportedTargetHandle {
        connection_id: String,
        handle: String,
    },
    #[error("patch interface entry \"{key}\" is duplicated")]
    DuplicateInterfaceKey { key: String },
    #[error("graph node \"{node_id}\" is missing a valid data.type")]
    InvalidGraphNodeType { node_id: String },
    #[error("invalid patch: {0}")]
    Invalid(String),
}

pub type Result<T> = core::result::Result<T, PatchError>;
