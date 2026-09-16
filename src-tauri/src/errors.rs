use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "type", content = "details")]
pub enum SkillSyncError {
    #[error("Błąd systemu plików: {0}")]
    FileSystem(String),

    #[error("Błąd operacji Git: {message}")]
    Git { code: i32, message: String },

    #[error("Brak uprawnień do katalogu: {path}")]
    PermissionDenied { path: String },

    #[error("Katalog roboczy zawiera niezacommitowane zmiany w śledzonych plikach")]
    WorktreeDirty,

    #[error("Niepoprawny format manifestu skill: {0}")]
    InvalidManifest(String),

    #[error("Nieobsługiwany automatyczny sposób aktualizacji: {0}")]
    UnsupportedUpdateMethod(String),

    #[error("Weryfikacja integralności po aktualizacji zakończona niepowodzeniem: {0}")]
    IntegrityCheckFailed(String),

    #[error("Błąd przywracania punktu zapasowego (rollback): {0}")]
    RollbackFailed(String),

    #[error("Przekroczono limit czasu operacji sieciowej ({0}s)")]
    NetworkTimeout(u64),

    #[error("Błąd konfiguracji: {0}")]
    Config(String),
}

impl From<std::io::Error> for SkillSyncError {
    fn from(err: std::io::Error) -> Self {
        SkillSyncError::FileSystem(err.to_string())
    }
}

impl From<git2::Error> for SkillSyncError {
    fn from(err: git2::Error) -> Self {
        SkillSyncError::Git {
            code: err.raw_code(),
            message: err.message().to_string(),
        }
    }
}
