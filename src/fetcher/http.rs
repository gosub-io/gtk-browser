use std::error::Error;
pub use thiserror::Error;
use url::Url;
use fetcher::HttpFetcher;

pub mod fetcher;
pub mod request;
pub mod response;
pub mod http;
mod agents;


// These defines the actual HTTP fetcher that will be used by the engine. This is based on the
// feature flags "http-agent-ureq" and "http-agent-reqwest". There must always be at most one
// of these features enabled.

// check if only one of these are enabled
#[cfg(all(feature = "http-agent-ureq", feature = "http-agent-reqwest"))]
compile_error!("Both 'http-agent-ureq' and 'http-agent-reqwest' features cannot be enabled at the same time.");

#[cfg(not(any(feature = "http-agent-ureq", feature = "http-agent-reqwest")))]
compile_error!(
    "No HTTP agent feature is enabled. Please enable one of: 'http-agent-ureq' or 'http-agent-reqwest'."
);

#[cfg(feature = "http-agent-ureq")]
pub type CompleteHttpFetcher = HttpFetcher<agents::ureq::UreqAgent>;

#[cfg(feature = "http-agent-reqwest")]
pub type CompleteHttpFetcher = HttpFetcher<agents::reqwest::ReqwestAgent>;



#[derive(Error, Debug)]
pub enum HttpError {
    #[error("{0}")]
    AgentError(#[from] Box<dyn Error + Send + 'static>),
    #[error("Timeout on: {0}")]
    Timeout(Url),
    #[error("Too many redirects: {0}")]
    TooManyRedirects(Url),
    #[error("Certificate error on: {0}")]
    Certificate(Url),
    #[error("Unknown error")]
    UnknownError,
}

