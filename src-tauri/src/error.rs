use thiserror::Error;

#[derive(Debug, Error)]
pub enum NexusError {
    #[error("Window creation failed: {0}")]
    WindowCreationFailed(String),

    #[error("Window not found: {0}")]
    WindowNotFound(String),

    #[error("Event delivery failed: {0}")]
    EventDeliveryFailed(String),

    #[error("Workspace serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Workspace deserialization failed: {0}")]
    DeserializationFailed(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Tauri error: {0}")]
    Tauri(String),
}

impl From<tauri::Error> for NexusError {
    fn from(e: tauri::Error) -> Self {
        NexusError::Tauri(e.to_string())
    }
}

// Needed for Tauri commands to return errors as strings to the frontend
impl serde::Serialize for NexusError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
