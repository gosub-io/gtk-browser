use crate::fetcher::http::HttpError;
use crate::fetcher::http::request::HttpRequest;
use crate::fetcher::http::response::HttpResponse;

#[cfg(all(feature = "http-agent", not(any(feature = "http-agent-reqwest", feature = "http-agent-ureq"))))]
compile_error!(
    "Feature 'http-agent' is enabled, but no HTTP library sub-feature is selected. \
     Choose one of: 'http-agent-reqwest' or 'http-agent-ureq'."
); //TODO: remove this and let the confiugration decide what to use

#[cfg(feature = "http-agent-reqwest")]
pub mod reqwest;

#[cfg(feature = "http-agent-ureq")]
pub mod ureq;

// The HTTPRequest agent is the actual library that will be used to make the request. We use
// dedicated libraries like reqwest, ureq, hyper, or surf to make the actual request and return
// it back into a more generic Gosub Http Request/Response format that is used by the engine.
pub trait HttpRequestAgent {
    /// Create a new request agent
    fn new() -> Self;

    /// Executes the given request and returns the response or an error
    async fn execute(&self, req: HttpRequest) -> Result<HttpResponse, HttpError>;
}


