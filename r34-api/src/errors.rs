use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid root url: {0}")]
    InvalidRootUrl(reqwest::Error),

    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("failed writing image: {0}")]
    OutputFile(#[from] io::Error),

    #[error("malformed credentials")]
    MalformedCredentials,

    #[error("missing credentials: {0}")]
    MissingCredentials(&'static str),
}
