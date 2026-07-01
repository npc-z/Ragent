use thiserror::Error;

#[derive(Error, Debug)]
pub enum RagentError {
    #[error("Environment variable `{0}` not set. Please set it in your .env file.")]
    EnvVarMissing(String),

    #[error("API request failed: {0}")]
    ApiRequest(String),

    #[error("API returned error status {status}: {body}")]
    ApiStatus { status: u16, body: String },

    #[error("Failed to parse {llm} API response: {e}")]
    ApiParse { llm: String, e: String },

    #[error("{0}")]
    ClientBuild(String),

    #[error("No response message from API (empty choices)")]
    EmptyResponse,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Path escapes workspace: {0}")]
    PathEscape(String),

    #[error("Path is not a file: {0}")]
    PathNotAFile(String),

    #[error("Path not exist: {0}")]
    PathNotExist(String),

    #[error("Text not found at path: {0}")]
    TextNotFound(String),
}
