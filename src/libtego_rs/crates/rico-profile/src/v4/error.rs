// std

// extern

// internal

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to open database: {0}")]
    DatabaseOpenFailure(#[source] rusqlite::Error),

    #[error("failed to update pragma: {0}")]
    PragmaUpdateFailure(#[source] rusqlite::Error),

    #[error("failed to execute statement: {0}")]
    StatementExecuteFailure(#[source] rusqlite::Error),

    #[error("failed to query row(s): {0}")]
    QueryFailure(#[source] rusqlite::Error),

    #[error("column get failure: {0}")]
    ColumnGetFailure(#[source] rusqlite::Error),

    #[error("invalid semantic version: {0}.{1}.{2}")]
    InvalidSemanticVersion(i64, i64, i64),

    #[error("unknown profile version: {0}")]
    UnknownProfileVersion(crate::v4::profile::Version),

    #[error("not implemented")]
    NotImplemented,
}
