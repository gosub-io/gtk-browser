use crate::fetcher::async_stream::AsyncStream;
use fetcher::HttpFetcher;
use std::error::Error;
use std::fmt::Debug;
pub use thiserror::Error;
use url::Url;

mod agents;
pub mod fetcher;
pub mod request;
pub mod response;

// These defines the actual HTTP fetcher that will be used by the engine. This is based on the
// feature flags "http-agent-ureq" and "http-agent-reqwest". There must always be at most one
// of these features enabled.

// check if only one of these are enabled
#[cfg(all(feature = "http-agent-ureq", feature = "http-agent-reqwest"))]
compile_error!("Both 'http-agent-ureq' and 'http-agent-reqwest' features cannot be enabled at the same time.");

#[cfg(not(any(feature = "http-agent-ureq", feature = "http-agent-reqwest")))]
compile_error!("No HTTP agent feature is enabled. Please enable one of: 'http-agent-ureq' or 'http-agent-reqwest'.");

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

pub const GOSUB_USERAGENT_STRING: &str =
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; Wayland; rv:133.0) Gecko/20100101 Gosub/0.1 Firefox/133.0";

pub enum HttpBody {
    /// A reader that can stream data (e.g. file, network)
    Reader(AsyncStream),
    /// Body is empty
    Empty,
}

impl Clone for HttpBody {
    fn clone(&self) -> Self {
        Self::Empty
    }
}

impl Debug for HttpBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpBody::Reader(_) => write!(f, "HttpBody::Reader"),
            HttpBody::Empty => write!(f, "HttpBody::Empty"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Options,
    Head,
    Patch,
    Trace,
    Connect,
    Custom(String),
}

impl HttpMethod {
    fn from_str(method: &str) -> Self {
        match method.to_ascii_uppercase().as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "OPTIONS" => HttpMethod::Options,
            "HEAD" => HttpMethod::Head,
            "PATCH" => HttpMethod::Patch,
            "TRACE" => HttpMethod::Trace,
            _ => HttpMethod::Custom(method.to_string()),
        }
    }

    fn as_str(&self) -> &str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Head => "HEAD",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Trace => "TRACE",
            HttpMethod::Connect => "CONNECT",
            HttpMethod::Custom(s) => s.as_str(),
        }
    }
}
