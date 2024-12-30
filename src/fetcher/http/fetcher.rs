use url::Url;
use crate::fetcher::http::agents::HttpRequestAgent;
use crate::fetcher::http::http::HttpMethod;
use crate::fetcher::http::request::HttpRequest;
use crate::fetcher::http::response::HttpResponse;
use crate::fetcher::http::HttpError;

pub struct Fetcher<R: HttpRequestAgent> {
    // Base URL to resolve all relative URLs from
    base_url: Url,
    /// Actual library that does the HTTP fetching
    agent: R,
    /// Additional middleware (logging, caching, security?)
    middleware: Option<String>
}

impl<R: HttpRequestAgent> Fetcher<R> {
    /// Creates a new HTTP fetcher for the given baseUrl
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            agent: R::new(),
            middleware: None,
        }
    }

    pub async fn fetch(&self, method: HttpMethod, url: Url) -> Result<HttpResponse, HttpError> {
        let req = HttpRequest::new(method, url);
        self.agent.execute(req).await
    }

    pub async fn fetch_with_request(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
        self.agent.execute(request).await
    }
}
