use std::fmt;
use std::fmt::{Debug, Formatter};
use crate::tab::TabId;

pub enum Message {
    // Open a new tab, and load a URL
    OpenTab(String),
    /// Sent when we need to load a new url into a tab
    LoadUrl(TabId, String),

    /// Sent when a favicon has been loaded for tab X
    FaviconLoaded(TabId, Vec<u8>),
    // Sent when a URL has been loaded for tab X
    UrlLoaded(TabId, String),
    // Refresh tabs
    RefreshTabs(),

    // Pins a tab
    PinTab(TabId),
    // Unpins a tab
    UnpinTab(TabId),

    // Single message to print in the log
    Log(String),
}

impl Debug for Message {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Message::OpenTab(url) => write!(f, "OpenTab({})", url),
            Message::LoadUrl(tab_id, url) => write!(f, "LoadUrl({:?}, {})", tab_id, url),
            Message::FaviconLoaded(tab_id, favicon) => write!(f, "FaviconLoaded({:?}, {} bytes)", tab_id, favicon.len()),
            Message::UrlLoaded(tab_id, content) => write!(f, "UrlLoaded({:?}, {} bytes)", tab_id, content.len()),
            Message::RefreshTabs() => write!(f, "RefreshTabs()"),
            Message::Log(msg) => write!(f, "Log({})", msg),
            Message::PinTab(tab_id) => write!(f, "PinTab({:?})", tab_id),
            Message::UnpinTab(tab_id) => write!(f, "UnpinTab({:?})", tab_id),
        }
    }
}