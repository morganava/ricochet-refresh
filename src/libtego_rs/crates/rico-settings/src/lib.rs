pub mod common;
pub mod v3;
pub mod v4;
#[derive(thiserror::Error, Debug)]
pub enum Error {
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
