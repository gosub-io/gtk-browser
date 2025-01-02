use crate::fetcher::http::agents::HttpRequestAgent;
use crate::fetcher::http::request::HttpRequest;
use crate::fetcher::http::response::HttpResponse;
use crate::fetcher::http::HttpError;
use url::Url;

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
