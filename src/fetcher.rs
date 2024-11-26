use log::info;
use reqwest::{Client, Error, Response};
use std::time::Duration;

const GOSUB_USERAGENT_STRING: &str = "Mozilla/5.0 (X11; Linux x86_64; Wayland; rv:1.0) Gecko/20231106 Gosub/0.1 Firefox/89.0";

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

    info!("fetching url {}", url);
    let client = Client::builder()
        .user_agent(GOSUB_USERAGENT_STRING)
        .timeout(Duration::from_secs(5))
        .build()?;

    client.get(url).send().await
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
