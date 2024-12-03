use log::info;
use reqwest::header::HeaderMap;
use reqwest::{Client, Error, Response};
use std::time::Duration;

const GOSUB_USERAGENT_STRING: &str = "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; Wayland; rv:133.0) Gecko/20100101 Gosub/0.1 Firefox/133.0";
const FIREFOX_USERAGENT_STRING: &str = "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:133.0) Gecko/20100101 Firefox/133.0";

/// Fetches the (binary) body of a URL and returns it as a Vec<u8>
pub async fn fetch_url_body(url: &str) -> Result<Vec<u8>, Error> {
    match fetch_url(url).await {
        Ok(response) => {
            let body = response.bytes().await?.to_vec();
            Ok(body)
        }
        Err(e) => Err(e),
    }
}

/// Fetches an URL and returns the response
pub async fn fetch_url(url: &str) -> Result<Response, Error> {
    // info!("sleeping 3 seconds before fetch_url({})", url);
    // sleep(Duration::from_secs(3)).await;

    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", GOSUB_USERAGENT_STRING.parse().unwrap());
    headers.insert(
        "Accept",
        "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".parse().unwrap(),
    );
    headers.insert("Accept-Language", "en-US,en;q=0.5".parse().unwrap());
    // headers.insert("Connection", "keep-alive".parse().unwrap());
    headers.insert("DNT", "1".parse().unwrap());

    info!("fetching url {}", url);
    let client = Client::builder()
        .user_agent(FIREFOX_USERAGENT_STRING)
        .timeout(Duration::from_secs(5))
        .use_rustls_tls() // For HTTP2
        .connect_timeout(Duration::from_secs(5))
        .connection_verbose(true)
        .read_timeout(Duration::from_secs(5))
        .brotli(true)
        .gzip(true)
        .deflate(true)
        .build()?;

    let request_builder = client.get(url).headers(headers);
    let request = request_builder.build()?;

    println!("Request Method: {:?}", request.method());
    println!("Request URL: {:?}", request.url());
    println!("Request Headers: {:#?}", request.headers());

    let response = client.execute(request).await?;

    // println!("Response Status: {:?}", response.status());
    // println!("Response Headers: {:#?}", response.headers());

    Ok(response)
}

/// Fetches the favicon from a URL and returns it as a Pixbuf
pub async fn fetch_favicon(url: &str) -> Vec<u8> {
    // info!("sleeping 3 seconds before fetch_favicon({})", url);
    // sleep(Duration::from_secs(3)).await;

    info!("fetching favicon from {}", url);
    let url = format!("{}{}", url, "/favicon.ico");
    let Ok(buf) = fetch_url_body(url.as_str()).await else {
        info!("Failed to fetch favicon from URL");
        return Vec::new();
    };

    buf
}
