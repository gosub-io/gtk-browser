use url::Url;
use crate::fetcher::gopher::GopherError;

pub struct GopherRequest {
    url: Url,
}

impl GopherRequest {
    pub fn new(url: Url) -> Self {
        Self {
            url,
        }
    }
}

pub struct GopherResponse {}

pub struct GopherFetcher {
    base_url: Url,
    // client: GopherRequestAgent,
    // middleware: Option<>
}

impl GopherFetcher {
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
        }
    }

    pub async fn fetch(&self, _request: GopherRequest) -> Result<GopherResponse, GopherError> {
        Ok(GopherResponse {})
    }
}
