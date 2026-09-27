use serde::Serialize;

/// Public errors deliberately exclude SQLite statements and record contents.
#[derive(Debug, thiserror::Error, Serialize, PartialEq, Eq)]
#[serde(tag = "code", content = "message", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Error {
    #[error("Sign in again to continue.")]
    Unauthenticated,
    #[error("Your account does not have permission for this operation.")]
    Forbidden,
    #[error("The username or password is incorrect, or sign-in is temporarily limited.")]
    InvalidCredentials,
    #[error("{0}")]
    Validation(String),
    #[error("This record no longer exists or is unavailable.")]
    NotFound,
    #[error("This operation conflicts with an existing record. Refresh and try again.")]
    Conflict,
    #[error("The database could not complete this operation. Keep your data and use recovery.")]
    Storage,
    #[error("The database schema is unsupported or has changed unexpectedly. Use recovery.")]
    Schema,
    #[error("Setup has already been completed.")]
    AlreadyConfigured,
    #[error("Complete activation before setting up the clinic.")]
    ActivationRequired,
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self {
        match value {
            rusqlite::Error::QueryReturnedNoRows => Self::NotFound,
            rusqlite::Error::SqliteFailure(ref e, _)
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Self::Conflict
            }
            _ => Self::Storage,
        }
    }
}

pub(crate) fn invalid(message: &str) -> Error {
    Error::Validation(message.to_owned())
}
