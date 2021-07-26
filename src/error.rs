#[derive(Debug, thiserror::Error)]
/// All errors that can happen
pub enum Error {
    #[error("Json error: {0}")]
    Json(#[from] serde_json::error::Error),

    #[error("Error writing file: {0}")]
    Write(#[from] std::io::Error),

    #[error("Request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Invalid credentials can't login")]
    InvalidCredentials,

    #[error("XML parsing error: {0}")]
    Xml(#[from] quick_xml::DeError),

    #[error("Rtmp is offline: {0}")]
    RtmpDown(String),

    #[error("No inventories found")]
    NoInventoriesFound,

    #[error("No units found")]
    NoUnitsFound,

    #[error("Status not available")]
    StatusNotAvailable,

    #[error("Not enough permissions to use command")]
    NotEnoughPermissions,
}
