use std::fmt::Debug;
use crate::fetcher::async_stream::AsyncStream;

pub const GOSUB_USERAGENT_STRING: &str = "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; Wayland; rv:133.0) Gecko/20100101 Gosub/0.1 Firefox/133.0";

pub enum HttpBody {
    /// A reader that can stream data (e.g. file, network)
    Reader(AsyncStream),
    /// Body is empty
    Empty,
}


impl Clone for HttpBody {
    fn clone(&self) -> Self {
        Self::Empty
    }
}

impl Debug for HttpBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpBody::Reader(_) => write!(f, "HttpBody::Reader"),
            HttpBody::Empty => write!(f, "HttpBody::Empty"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Options,
    Head,
    Patch,
    Trace,
    Connect,
    Custom(String),
}

impl HttpMethod {
    fn from_str(method: &str)-> Self {
        match method.to_ascii_uppercase().as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "OPTIONS" => HttpMethod::Options,
            "HEAD" => HttpMethod::Head,
            "PATCH" => HttpMethod::Patch,
            "TRACE" => HttpMethod::Trace,
            _ => HttpMethod::Custom(method.to_string()),
        }
    }

    fn as_str(&self) -> &str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Head => "HEAD",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Trace => "TRACE",
            HttpMethod::Connect => "CONNECT",
            HttpMethod::Custom(s) => s.as_str(),
        }
    }
}
