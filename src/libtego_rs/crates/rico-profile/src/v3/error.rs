#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    TorCrypto(#[from] tor_interface::tor_crypto::Error),

    #[error(transparent)]
    SerdeJSON(#[from] serde_json::Error),
}
