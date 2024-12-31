use url::Url;
use crate::fetcher::http::agents::HttpRequestAgent;
use crate::fetcher::http::HttpError;
use crate::fetcher::http::request::HttpRequest;
use crate::fetcher::http::response::HttpResponse;

pub struct UreqAgent;

impl HttpRequestAgent for UreqAgent {
    fn new() -> Self {
        Self
    }

    async fn execute(&self, _req: HttpRequest) -> Result<HttpResponse, HttpError> {
        // ureq::AgentBuilder::new()
        Err(HttpError::Timeout(Url::parse("http://example.com").unwrap()))
    }
}