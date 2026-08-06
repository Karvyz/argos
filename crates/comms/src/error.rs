#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("communication error: {0}")]
    Transport(#[from] zenoh::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] postcard::Error),
    #[error("invalid {topic} payload: {reason}")]
    InvalidPayload {
        topic: &'static str,
        reason: &'static str,
    },
}
