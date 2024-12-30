use gosub_engine::prelude::ModuleConfiguration;
use url::Url;

struct FtpFetcher {
    base_url: Url,
    // client: FtpRequestAgent,
    // middleware: aMiddlewareStuff<>
}

impl FtpFetcher {
    fn new(base_url: Url) {

    }

    fn fetch(&self, method: FtpMethod, url: Url) -> Result<FtpResponse, Self::Error> {
    }
}
