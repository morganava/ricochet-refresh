// std

// extern

// internal

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ComponentRange(#[from] time::error::ComponentRange),

    #[error(transparent)]
    TorCrypto(#[from] tor_interface::tor_crypto::Error),

    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),

    #[error("invalid password")]
    InvalidPassword,

    #[error(transparent)]
    RicoProtocolV4(#[from] rico_protocol::v4::Error),

    #[error("could not convert '{0}' to type {1}")]
    TypeConversionFailed(String, &'static str),

    #[error("invalid semantic version: {0}.{1}.{2}")]
    InvalidSemanticVersion(i64, i64, i64),

    #[error("unknown profile version: {0}")]
    UnknownProfileVersion(crate::v4::profile::Version),

    #[error("not implemented")]
    NotImplemented,
}
