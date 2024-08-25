pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid root url: {0}")]
    InvalidRootUrl(reqwest::Error),

    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
}
