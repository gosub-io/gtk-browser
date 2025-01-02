pub use thiserror::Error;

pub mod fetcher;

#[derive(Error, Debug)]
pub enum FtpError {
    #[error("Too many ftp stuff happening: {0}")]
    FtpError(String),
}
