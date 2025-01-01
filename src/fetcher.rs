use log::error;
use url::Url;
use thiserror::Error;

#[cfg(not(feature = "proto-http"))]
compile_error!("Feature 'proto-http' cannot be disabled.");

#[cfg(feature = "proto-http")]
mod http;
#[cfg(feature = "proto-ftp")]
mod ftp;
#[cfg(feature = "proto-gopher")]
mod gopher;

mod async_stream;


#[derive(Error, Debug)]
pub enum FetcherError {
    #[cfg(feature = "proto-http")]
    #[error("http error: {0}")]
    Http(#[from] http::HttpError),

    #[cfg(feature = "proto-ftp")]
    #[error("ftp error: {0}")]
    Ftp(#[from] ftp::FtpError),

    #[cfg(feature = "proto-gopher")]
    #[error("gopher error: {0}")]
    Gopher(#[from] gopher::GopherError),

    #[error("unsupported scheme: {0}")]
    UnsupportedScheme(String),
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
}


#[cfg(feature = "proto-http")]
pub use crate::fetcher::http::{
    CompleteHttpFetcher,
    response::HttpResponse,
    request::HttpRequest,
    http::HttpMethod,
};

#[cfg(feature = "proto-ftp")]
pub use crate::fetcher::ftp::{
    fetcher::FtpFetcher,
    fetcher::FtpRequest,
    fetcher::FtpResponse
};

#[cfg(feature = "proto-gopher")]
pub use crate::fetcher::gopher::{
    fetcher::GopherFetcher,
    fetcher::GopherRequest,
    fetcher::GopherResponse
};
use crate::fetcher::http::http::HttpBody;

enum Response {
    #[cfg(feature = "proto-http")]
    Http(HttpResponse),
    #[cfg(feature = "proto-ftp")]
    Ftp(FtpResponse),
    #[cfg(feature = "proto-gopher")]
    Gopher(GopherResponse),
    // #[cfg(feature = "proto-file")]
    // File(FileResponse),
    // #[cfg(feature = "proto-irc")]
    // Irc(IrcResponse),
}

pub struct Fetcher {
    #[cfg(feature = "proto-http")]
    http_fetcher: CompleteHttpFetcher,  // This is the fetcher with the compiled request agent (ie: HttpAgent<ReqwestAgent>)
    #[cfg(feature = "proto-ftp")]
    ftp_fetcher: FtpFetcher,
    #[cfg(feature = "proto-gopher")]
    gopher_fetcher: GopherFetcher,
}

impl Fetcher {
    pub fn protocols_implemented() -> Vec<String> {
        let mut protocols = vec![];

        #[cfg(feature = "proto-http")]
        protocols.push("http".to_string());
        #[cfg(feature = "proto-ftp")]
        protocols.push("ftp".to_string());
        // #[cfg(feature = "proto-file")]
        // protocols.push("file".to_string());
        // #[cfg(feature = "proto-irc")]
        // protocols.push("irc".to_string());
        #[cfg(feature = "proto-gopher")]
        protocols.push("gopher".to_string());

        protocols
    }

    pub fn new(base_url: Url) -> Self {
        Fetcher {
            #[cfg(feature = "proto-http")]
            http_fetcher: CompleteHttpFetcher::new(base_url.clone()),
            #[cfg(feature = "proto-ftp")]
            ftp_fetcher: FtpFetcher::new(base_url.clone()),
            #[cfg(feature = "proto-gopher")]
            gopher_fetcher: GopherFetcher::new(base_url.clone()),
        }
    }

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
                let request = FtpRequest::new(url);
                match self.ftp_fetcher.fetch(request).await {
                    Ok(response) => Ok(Response::Ftp(response)),
                    Err(e) => Err(FetcherError::Ftp(e)),
                }
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
                match self.gopher_fetcher.fetch(request).await {
                    Ok(response) => Ok(Response::Gopher(response)),
                    Err(e) => Err(FetcherError::Gopher(e)),
                }
            }
            _ => {
                error!("Unsupported scheme: {}", scheme);
                Err(FetcherError::UnsupportedScheme(scheme.to_string()))
            }
        }
    }
}

pub async fn fetch_favicon(url: &str) -> Vec<u8> {
    let Ok(url) = Url::parse(url) else {
        return Vec::new();
    };

    let url = url.join("/favicon.ico").unwrap();

    // This should be a method in Fetcher... it's a lot of boilerplate for fetching a simple favicon
    let fetcher = Fetcher::new(url.clone());
    match fetcher.fetch(url).await {
        // There was a correct response
        Ok(response) => {
            match response {
                Response::Http(http_response) => {
                    if http_response.head().status_code() == 200 {
                        match http_response.body() {
                            HttpBody::Reader(reader) => {
                                match reader.to_vec().await {
                                    Ok(data) => data,
                                    Err(e) => {
                                        error!("Failed to fetch favicon from URL: {:?}", e);
                                        Vec::new()
                                    }
                                }
                            }
                            HttpBody::Empty => Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                }
                #[allow(unreachable_patterns)]
                _ => Vec::new()
            }
        }
        Err(e) => {
            error!("Failed to fetch favicon from URL: {:?}", e);
            Vec::new()
        }
    }
}

pub async fn fetch_url_body(url_str: &str) -> Result<Vec<u8>, FetcherError> {
    let Ok(url) = Url::parse(url_str) else {
        return Err(FetcherError::InvalidUrl(url_str.to_string()));
    };

    let fetcher = Fetcher::new(url.clone());
    match fetcher.fetch(url).await {
        // There was a correct response
        Ok(response) => {
            match response {
                // It is an HTTP response
                Response::Http(http_response) => {
                    // It is a 200 OK response
                    if http_response.head().status_code() == 200 {
                        match http_response.body() {
                            // We've got a body
                            HttpBody::Reader(reader) => {
                                match reader.to_vec().await {
                                    // We've got the body as a Vec<u8>
                                    Ok(data) => Ok(data),
                                    Err(e) => {
                                        error!("Failed to fetch body from URL: {:?}", e);
                                        Err(FetcherError::Http(http::HttpError::UnknownError))
                                    }
                                }
                            }
                            HttpBody::Empty => Ok(Vec::new())
                        }
                    } else {
                        Err(FetcherError::Http(http::HttpError::UnknownError))
                    }
                }
                #[allow(unreachable_patterns)]
                _ => {
                    error!("Unsupported response type. We expected a HTTP response");
                    Err(FetcherError::Http(http::HttpError::UnknownError))
                }
            }
        }
        Err(e) => {
            Err(e)
        }
    }
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
