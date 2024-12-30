use log::error;
use url::Url;
use thiserror::Error;

#[cfg(not(feature = "proto-http"))]
compile_error!("Feature 'proto-http' cannot be disabled.");

#[cfg(feature = "proto-http")]
mod http;

#[cfg(feature = "proto-gopher")]
mod gopher;
mod async_stream;

#[derive(Error, Debug)]
pub enum FetcherError {
    #[cfg(feature = "proto-http")]
    #[error("http error: {0}")]
    Http(#[from] http::HttpError),
    #[cfg(feature = "proto-gopher")]
    #[error("gopher error: {0}")]
    Gopher(#[from] gopher::GopherError),
    #[error("unsupported scheme")]
    UnsupportedScheme
}


#[cfg(feature = "proto-http")]
pub use crate::fetcher::http::{
    HttpFetcher,
    response::HttpResponse,
    request::HttpRequest,
    http::HttpMethod,
};

#[cfg(feature = "proto-gopher")]
pub use crate::fetcher::gopher::{
    fetcher::GopherFetcher,
    fetcher::GopherRequest,
    fetcher::GopherResponse
};

enum Response {
    #[cfg(feature = "proto-http")]
    Http(HttpResponse),
    #[cfg(feature = "proto-gopher")]
    Gopher(GopherResponse),
    // #[cfg(feature = "proto-ftp")]
    // Ftp(FtpResponse),
    // #[cfg(feature = "proto-file")]
    // File(FileResponse),
    // #[cfg(feature = "proto-irc")]
    // Irc(IrcResponse),
}

struct Fetcher {
    #[cfg(feature = "proto-http")]
    http_fetcher: HttpFetcher,
    #[cfg(feature = "proto-gopher")]
    gopher_fetcher: GopherFetcher,
}

impl Fetcher {
    async fn fetch(&self, url: Url) -> Result<Response, FetcherError> {
        let scheme = url.scheme();

        match scheme {
            #[cfg(feature = "proto-http")]
            "https" | "http" => {
                let request = HttpRequest::new(HttpMethod::Get, url);
                match self.http_fetcher.fetch_with_request(request).await {
                    Ok(response) => Ok(Response::Http(response)),
                    Err(e) => Err(FetcherError::Http(e)),
                }
            }
            #[cfg(feature = "proto-ftp")]
            "ftp" => {
                // let request = FtpRequest::new(FtpMethod::Get, url);
                // self.ftp_fetcher.fetch(request)
            }
            // "file" => {
            //     self.file_fetcher.fetch(url)
            // }
            // "irc" => {
            //     self.irc_fetcher.fetch(url)
            // }
            #[cfg(feature = "proto-gopher")]
            "gopher" => {
                let request = GopherRequest::new(url);
                self.gopher_fetcher.fetch(request)
            }
            _ => {
                error!("Unsupported scheme: {}", scheme);
                Err(FetcherError::UnsupportedScheme)
            }
        }
    }
}

pub async fn fetch_favicon(_url: &str) -> Vec<u8> {
    Vec::new()
}

pub async fn fetch_url_body(_url: &str) -> Result<Vec<u8>, FetcherError> {
    Ok(Vec::new())
}

// fn get_favicon() {
//     let tx = fetcher.fetch("favicon.ico");
//     if tx.is_error() {
//         ....
//     }
//
//
//     match fetcher.fetch("favicon.ico") {
//         Ok(tx) => {
//             print!(tx.url);
//         }
//         Err(e) => {
//             print!(e.transaction);
//         }
//     }
//
// }


// fn clone_request(req: &Request) -> Request {
//     let mut req = Request::new(req.method().clone(), req.url().clone());
//     *req.timeout_mut() = req.timeout().copied();
//     *req.headers_mut() = req.headers().clone();
//     *req.version_mut() = req.version();
//     *req.body_mut() = *req.body().clone();
//
//     req
// }
//
// /// Internal Gosub HTTP methods. It has support for custom methods as well if needed.
//
//
// /// Create a default request object with the given method and URL
// fn create_request(method: HttpMethod, url: Url) -> reqwest::Request {
//     // We deal with headers here, but this should be more flexible
//     let mut headers = HeaderMap::new();
//     headers.insert("User-Agent", GOSUB_USERAGENT_STRING.parse().unwrap());
//     headers.insert(
//         "Accept",
//         "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".parse().unwrap(),
//     );
//     headers.insert("Accept-Language", "en-US,en;q=0.5".parse().unwrap());
//     // headers.insert("Connection", "keep-alive".parse().unwrap());
//     headers.insert("DNT", "1".parse().unwrap());
//
//     // Cookies  should be dealt with more flexible
//     let jar = match SqliteStorage::new("./gosub_cookies.db") {
//         Ok(store) => {
//             info!("successfully created SqliteStorage");
//             Some(CookieJar::new(Arc::new(Mutex::new(store))))
//         }
//         Err(e) => {
//             info!("failed to create SqliteStorage: {:?}", e);
//             None
//         }
//     };
//
//     // Create the actual client that will handle the request
//     let mut builder = reqwest::Client::builder()
//         .user_agent(GOSUB_USERAGENT_STRING)
//         .timeout(Duration::from_secs(5))
//         .use_rustls_tls() // For HTTP2
//         .connect_timeout(Duration::from_secs(5))
//         .connection_verbose(true)
//         .read_timeout(Duration::from_secs(5))
//         .brotli(true)
//         .gzip(true)
//         .deflate(true);
//
//     match jar {
//         Some(jar) => {
//             builder = builder.cookie_provider(Arc::new(jar));
//         }
//         None => {
//             info!("no cookie jar");
//         }
//     }
//     let client = builder.build()?;
//
//     let request_builder = client.request(method.into(), url).headers(headers.clone());
//     let request = request_builder.build()?;
//
//     request
// }
//
// /// Fetches the favicon from a URL and returns it as a Pixbuf
// pub async fn fetch_favicon(url: &str) -> Vec<u8> {
//     info!("fetching favicon from {}", url);
//
//     match Url::from_str(format!("{}{}", url, "/favicon.ico").as_str()) {
//         Ok(url) => {
//             let mut transaction = HttpTransaction::new(HttpMethod::Get, url);
//
//             match transaction.execute().await {
//                 Ok(_) => {
//                     if transaction.is_success() {
//                         transaction.response.unwrap().bytes().await.unwrap().to_vec()
//                     } else {
//                         Vec::new()
//                     }
//                 }
//                 Err(e) => {
//                     warn!("Failed to fetch favicon from URL: {:?}", e);
//                     Vec::new()
//                 }
//             }
//         }
//         Err(e) => {
//             warn!("Failed to parse URL: {:?}", e);
//             Vec::new()
//         }
//     }
// }
//
// /// Fetches the (binary) body of a URL through a GET request and returns it as a Vec<u8>
// pub async fn fetch_url_body(url: &str) -> Result<Vec<u8>, Error> {
//     let Some(url) = Url::from_str(url) else {
//         return Err(Error::new(reqwest::StatusCode::BAD_REQUEST, "Invalid URL"));
//     };
//
//     let mut transaction = HttpTransaction::new(HttpMethod::Get, url);
//     match transaction.execute().await {
//         Ok(_) => {
//             if transaction.is_success() {
//                 let body = transaction.response.unwrap().bytes().await.unwrap().to_vec();
//                 Ok(body)
//             }
//         }
//         Err(e) => {
//             warn!("Failed to fetch body from URL: {:?}", e);
//             Err(Error::new(reqwest::StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch body from URL"))
//         }
//     }
// }
