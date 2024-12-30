use crate::fetcher::http::agents::HttpRequestAgent;
use crate::fetcher::http::request::HttpRequest;
use crate::fetcher::http::response::HttpResponse;

pub struct UreqAgent;

impl HttpRequestAgent for UreqAgent {
    type Error = ();

    fn new() -> Self {
        Self
    }

    fn execute(&self, req: HttpRequest) -> Result<HttpResponse, Self::Error> {
        ureq::AgentBuilder::new()

    }
}