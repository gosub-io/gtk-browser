use std::io::Read;

pub const GOSUB_USERAGENT_STRING: &str = "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; Wayland; rv:133.0) Gecko/20100101 Gosub/0.1 Firefox/133.0";

#[derive(Debug, Clone, PartialEq)]
pub enum HttpBody {
    /// A chunk of bytes in memory
    Bytes(Vec<u8>),
    /// A reader that can stream data (e.g. file, network)
    Reader(Box<dyn Read + Send + 'static>),
    /// No body given
    Empty,
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
