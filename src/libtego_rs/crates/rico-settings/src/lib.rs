pub mod common;
pub mod v3;
pub mod v4;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Not Implemented")]
    NotImplemented,

    #[error("Conversion failed: {0}")]
    ConversionFailed(&'static str),

    #[error(transparent)]
    BridgeLineError(#[from] tor_interface::censorship_circumvention::BridgeLineError),

    #[error(transparent)]
    ProxyConfigError(#[from] tor_interface::proxy::ProxyConfigError),

    #[error(transparent)]
    TorProviderError(#[from] tor_interface::tor_provider::Error),

    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
}

#[cfg(not(any(
    feature = "bundled-tor",
    feature = "external-tor",
    feature = "arti-client"
)))]
compile_error!("At least one of the features \"bundled-tor\", \"external-tor\", or \"arti-client\" must be enabled.");
