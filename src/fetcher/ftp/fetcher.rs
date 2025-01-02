use crate::fetcher::ftp::FtpError;
use url::Url;

pub struct FtpRequest {
    url: Url,
}

impl FtpRequest {
    pub fn new(url: Url) -> Self {
        Self { url }
    }
}

pub struct FtpResponse {}

pub struct FtpFetcher {
    base_url: Url,
    // client: FtpRequestAgent,
    // middleware: Option<>
}

impl FtpFetcher {
    pub fn new(base_url: Url) -> Self {
        Self { base_url }
    }

    pub async fn fetch(&self, _request: FtpRequest) -> Result<FtpResponse, FtpError> {
        Ok(FtpResponse {})
    }
}
