use std::error::Error;
pub use thiserror::Error;
use url::Url;
use fetcher::Fetcher;

pub mod fetcher;
pub mod request;
pub mod response;
pub mod http;
mod agents;


#[cfg(feature = "http-agent-ureq")]
type HttpFetcher = Fetcher<agents::ureq::UreqAgent>;

#[cfg(feature = "http-agent-reqwest")]
pub type HttpFetcher = Fetcher<agents::reqwest::ReqwestAgent>;


#[derive(Error, Debug)]
pub enum HttpError {
    #[error("{0}")]
    AgentError(#[from] Box<dyn Error + Send + 'static>),
    #[error("Timeout on: {0}")]
    Timeout(Url),
    #[error("Too many redirects: {0}")]
    TooManyRedirects(Url),
    #[error("Certificate error on: {0}")]
    Certificate(Url)
}

