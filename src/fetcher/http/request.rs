use crate::fetcher::http::{HttpBody, HttpMethod};
use std::collections::HashMap;
use url::Url;

/// Gosub HTTP Request
#[derive(Debug, Clone)]
pub struct HttpRequest {
    /// Method of the request (GET, POST, PUT, DELETE, etc)
    pub method: HttpMethod,
    /// Actual URL of the request
    pub url: Url,
    /// Additional query parameters
    pub query_params: HashMap<String, String>,
    /// Headers to send
    pub headers: HashMap<String, String>,
    /// Optional body that can be sent with the request (in case of POST, PUT, etc)
    pub body: HttpBody,
    /// Cookies to send with the request
    pub cookies: HashMap<String, String>,
}

impl HttpRequest {
    /// Create a new http request with the given method and URL
    pub fn new(method: HttpMethod, url: Url) -> Self {
        Self {
            method,
            url,
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: HttpBody::Empty,
            cookies: HashMap::new(),
        }
    }
}

/// Builder for HttpRequest
struct HttpRequestBuilder {
    request: HttpRequest,
}

impl HttpRequestBuilder {
    pub fn new(method: HttpMethod, url: Url) -> Self {
        Self {
            request: HttpRequest::new(method, url),
        }
    }

    pub fn query_param(&mut self, key: &str, value: &str) -> &mut Self {
        self.request.query_params.insert(key.to_string(), value.to_string());
        self
    }

    pub fn header(&mut self, key: &str, value: &str) -> &mut Self {
        self.request.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn cookie(&mut self, key: &str, value: &str) -> &mut Self {
        self.request.cookies.insert(key.to_string(), value.to_string());
        self
    }

    pub fn body(&mut self, body: HttpBody) -> &mut Self {
        self.request.body = body;
        self
    }

    pub fn build(&self) -> HttpRequest {
        self.request.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_http_request_builder() {
        let url = Url::parse("http://example.com").unwrap();
        let request = HttpRequestBuilder::new(HttpMethod::Get, url.clone())
            .query_param("key1", "value1")
            .query_param("key2", "value2")
            .header("Content-Type", "application/json")
            .cookie("session", "123456")
            .body(HttpBody::Empty)
            .build();

        assert_eq!(request.method, HttpMethod::Get);
        assert_eq!(request.url, url.clone());
        assert_eq!(request.query_params.len(), 2);
        assert_eq!(request.query_params.get("key1"), Some(&"value1".to_string()));
        assert_eq!(request.query_params.get("key2"), Some(&"value2".to_string()));
        assert_eq!(request.headers.len(), 1);
        assert_eq!(request.headers.get("Content-Type"), Some(&"application/json".to_string()));
        assert_eq!(request.cookies.len(), 1);
        assert_eq!(request.cookies.get("session"), Some(&"123456".to_string()));
    }
}
