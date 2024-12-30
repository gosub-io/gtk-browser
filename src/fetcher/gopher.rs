use std::error::Error;
use url::Url;
use crate::fetcher::http::Error;

pub mod fetcher;

#[derive(Error, Debug)]
pub enum GopherError {
    #[error("{0}")]
    AgentError(Box<dyn Error + Send + 'static>),
    Timeout(Url),
    GopherStuff(String),
}
