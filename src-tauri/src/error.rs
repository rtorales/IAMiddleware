use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Vault: {0}")]
    Vault(String),

    #[error("Provider: {0}")]
    Provider(String),

    #[error("Router: {0}")]
    Router(String),

    #[error("Database: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Budget exceeded for '{0}'")]
    BudgetExceeded(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Configuration: {0}")]
    Config(String),

    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
}

// Permite serializar AppError como string en IPC de Tauri
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
