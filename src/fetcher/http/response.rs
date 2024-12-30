use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use url::Url;
use crate::fetcher::http::http::HttpBody;

pub struct HttpResponse {
    /// Http header data
    header: ResponseHeader,
    /// Actual body data
    body: HttpBody,
}


#[derive(Debug, Clone, PartialEq)]
pub enum HttpVersion {
    Http09,
    Http10,
    Http11,
    Http2,
    Http3,
    Custom(String),
}


impl Display for HttpVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpVersion::Http09 => write!(f, "HTTP/0.9"),
            HttpVersion::Http10 => write!(f, "HTTP/1.0"),
            HttpVersion::Http11 => write!(f, "HTTP/1.1"),
            HttpVersion::Http2 => write!(f, "HTTP/2.0"),
            HttpVersion::Http3 => write!(f, "HTTP/3.0"),
            HttpVersion::Custom(s) => write!(f, "{}", s),
        }
    }
}

pub struct ResponseHeader {
    /// Http version (1.0, 1.1, 2.0)
    version: HttpVersion,
    /// Http status code (200, 404, 500)
    status_code: u16,
    /// Headers sent
    headers: HashMap<String, String>,
    /// Content length (if sent)
    content_length: Option<u64>,
    /// Cookies
    cookies: HashMap<String, String>,
    /// Final url (after redirection)
    url: Url,
    // extensions: ...
}

impl ResponseHeader {
    pub fn new(version: HttpVersion, status_code: u16, headers: HashMap<String, String>, content_length: Option<u64>, cookies: HashMap<String, String>, url: Url) -> Self {
        Self {
            version,
            status_code,
            headers,
            content_length,
            cookies,
            url,
        }
    }

    pub fn version(&self) -> HttpVersion {
        self.version.clone()
    }

    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn content_length(&self) -> Option<u64> {
        self.content_length
    }

    pub fn cookies(&self) -> &HashMap<String, String> {
        &self.cookies
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
}

impl HttpResponse {
    pub fn new(head: ResponseHeader, body: HttpBody) -> Self {
        Self {
            header: head,
            body,
        }
    }

    pub fn head(&self) -> &ResponseHeader {
        &self.header
    }

    pub fn body(&self) -> &HttpBody {
        &self.body
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_http_response() {
        let head = ResponseHeader {
            version: HttpVersion::Http11,
            status_code: 200,
            headers: HashMap::new(),
            content_length: None,
            cookies: HashMap::new(),
            url: Url::parse("https://example.com").unwrap(),
        };
        let body = HttpBody::Empty;
        let response = HttpResponse::new(head, body);
        assert_eq!(response.head().version, HttpVersion::Http11);
        assert_eq!(response.head().status_code, 200);
        assert_eq!(response.head().url.as_str(), "https://example.com");
    }
}