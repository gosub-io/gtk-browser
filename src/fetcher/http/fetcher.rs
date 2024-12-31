use url::Url;
use crate::fetcher::http::agents::HttpRequestAgent;
use crate::fetcher::http::http::HttpMethod;
use crate::fetcher::http::request::HttpRequest;
use crate::fetcher::http::response::HttpResponse;
use crate::fetcher::http::HttpError;

/// The HTTP fetcher is the main entry point for fetching HTTP resources (starting with https:// or http://).
/// It uses a HttpRequestAgent to actually perform the HTTP requests. All URLs are resolved relative to the base URL.
pub struct HttpFetcher<R: HttpRequestAgent> {
    // Base URL to resolve all relative URLs from
    base_url: Url,
    /// Actual library that does the HTTP fetching
    agent: R,
    /// Additional middleware (logging, caching, security?)
    middleware: Option<String>
}

impl<R: HttpRequestAgent> HttpFetcher<R> {
    /// Creates a new HTTP fetcher for the given baseUrl
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            agent: R::new(),
            middleware: None,
        }
    }

    /// Simple fetch with just method and URL.
    pub async fn fetch(&self, method: HttpMethod, url: Url) -> Result<HttpResponse, HttpError> {
        let req = HttpRequest::new(method, url);
        self.agent.execute(req).await
    }

    /// A more complex fetch with a request object.
    pub async fn fetch_with_request(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
        self.agent.execute(request).await
    }
}
