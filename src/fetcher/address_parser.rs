use std::collections::HashMap;
use lazy_static::lazy_static;
use url::Url;

/// There is a difference between a Gosub Address and a URL. A Gosub Address is something that a user
/// can enter on the address bar and can be resolved in different ways. For example, the user can enter
/// `example.com`, and the browser will resolve it to `https://example.com`. Or the user can enter
/// `source:example.com`, and the browser will resolve it to `https://example.com`, but will set the
/// render mode to "source". This can trigger the renderer to render the source code of the page instead
/// of rendering the page itself.
///
/// This module provides a parser that can parse a Gosub Address and convert it into a URL and a rendering
/// mode. The `GosubRenderMode` enum defines the different rendering modes that are supported.
///
/// The difference between render modes and schemes:
///
/// - Render modes are used to determine how the content should be rendered. For example, if the render
///   mode is set to "source", then the content should be rendered as source code. The actual URL that is
///   being rendered will have its own scheme (most likely https:// or http://).
/// - Schemes are used to determine how the URL should be resolved. For example, if the scheme is set to
///   "gopher", then the URL should be resolved as an gopher address and rendered as a gopher page
///   (provided the render mode is set to "rendered").
///
/// When functionality is wanted, for instance a custom calclator or a custom search engine, the user
/// could use a custom scheme resolver to resolve the URL to a custom page. For example, the user could
/// enter `calc:2+2` and the browser would resolve it to `calc:2+2` and render the result of the calculation
/// (4) instead of rendering the page.
///
/// Basically:
///    - Custom renderer: for a custom view of a URL
///    - Custom scheme: for custom functionality based on the URL

/*
Ultimately, we want this to be more flexible by using some kind of extension system so user-agent can
define their own and possibly handle it their own way (by hooking in with a callback system that will
output html that is rendered back by the engine. Something like this:

fn GosubEngine::add_custom_rendermode(url: Url, mode: GosubRenderMode, callback: Fn) -> String { }

Engine::add_custom_rendermode("reverse-view:", GosubRenderMode::Custom, |str| {
    // Fetch the URL in the normal way, and reverse the content
    let html = fetch_url(url);
    html.chars().rev().collect()
}

Call on the address bar with:   `reverse-view:https://www.gosub.io


*/


/// Defines the different rendering modes for a URL.
#[derive(Clone, Debug, PartialEq, Eq)]
enum GosubRenderMode {
    /// Rendered as-is (mostly HTML)
    Rendered,
    /// Special about pages
    About,
    /// Rendered as highlighted source
    Source,
    /// Rendered as raw source
    RawSource,
    /// Rendered as highlighted JSON
    Json,
    /// Rendered as highlighted XML
    Xml,
    /// Custom rendering mode (user-defined)
    Custom(String),
}

lazy_static! {
    static ref RENDER_MODES: HashMap<&'static str, GosubRenderMode> = {
        let mut m = HashMap::new();
        m.insert("source:", GosubRenderMode::Source);
        m.insert("view-source:", GosubRenderMode::Source);
        m.insert("about:", GosubRenderMode::About);
        m.insert("raw:", GosubRenderMode::RawSource);
        m.insert("raw-source:", GosubRenderMode::RawSource);
        m.insert("json:", GosubRenderMode::Json);
        m.insert("xml:", GosubRenderMode::Xml);
        m
    };
}

/// Default scheme to add when none is present
const DEFAULT_SCHEME: &str = "https://";

/// Allows to parse a Gosub address (something that you can type on the address bar), and
/// converts it into a URL and a rendering mode.
struct GosubAddressParser {}

impl GosubAddressParser {
    /// Parses the given address into a URL and a rendering mode.
    pub fn parse(address: &str) -> Result<(GosubRenderMode, Url), anyhow::Error> {
        if address.is_empty() {
            return Err(anyhow::anyhow!("Empty address"));
        }

        let mut address = address;
        let mut mode = GosubRenderMode::Rendered;
        for (name, value) in RENDER_MODES.iter() {
            if address.starts_with(name) {
                address = &address[name.len()..];
                mode = value.clone();
            }
        }

        // If scheme is about:, then we MUST have a URL that does not start with a scheme.as
        if mode == GosubRenderMode::About {
            match Url::parse(address) {
                Ok(url) => {
                    if url.scheme() == "about" {
                        return Ok((mode, url));
                    } else {
                        return Err(anyhow::anyhow!("Invalid about: URL: {}", url.scheme()));
                    }
                }
                Err(url::ParseError::RelativeUrlWithoutBase) => {
                    return Ok((mode, Url::parse(&format!("about:{}", address))?));
                }
                Err(_) => {
                    return Err(anyhow::anyhow!("Invalid about: URL: {}", address));
                }

            }
        }

        match Url::parse(address) {
            Ok(url) => Ok((mode, url)),
            Err(url::ParseError::RelativeUrlWithoutBase) => {
                // dbg!("NO MATCH: ", address);
                match Self::parse(&format!("{}{}", DEFAULT_SCHEME, address)) {
                    Ok((_, url)) => Ok((mode, url)),
                    Err(e) => Err(e),
                }
            }
            Err(e) => {
                Err(anyhow::anyhow!("Cannot parse URL: {}", e))
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! address_valid {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (address, expected_mode, expected_scheme, expected_host, expected_path) = $value;
                    let (found_mode, url) = GosubAddressParser::parse(address).expect(&format!("Failed to parse address: {}", address));
                    assert_eq!(
                        found_mode, expected_mode,
                        "Mode mismatch for address: {}. Expected: {:?}, Found: {:?}",
                        address, expected_mode, found_mode
                    );
                    assert_eq!(
                        url.scheme(), expected_scheme,
                        "Scheme mismatch for address: {}. Expected: {:?}, Found: {:?}",
                        address, expected_scheme, url.scheme()
                    );
                    assert_eq!(
                        url.host_str().unwrap_or(""), expected_host,
                        "Host mismatch for address: {}. Expected: {:?}, Found: {:?}",
                        address, expected_host, url.host_str().unwrap()
                    );
                    assert_eq!(
                        url.path(), expected_path,
                        "Path mismatch for address: {}. Expected: {:?}, Found: {:?}",
                        address, expected_path, url.path()
                    );
                }
            )*
        }
    }

    macro_rules! address_invalid {
    ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let address = $value;
                match GosubAddressParser::parse(address) {
                    Ok(_) => panic!("Expected an error but got success for address: {}", address),
                    Err(_) => assert!(true, "Successfully detected invalid address: {}", address),
                }
            }
        )*
    }
}

    address_valid! {
        test_1: ("https://example.com", GosubRenderMode::Rendered, "https", "example.com", "/"),
        test_2: ("example.com", GosubRenderMode::Rendered, "https", "example.com", "/"),
        test_3: ("http://example.com", GosubRenderMode::Rendered, "http", "example.com", "/"),
        test_4: ("http://example", GosubRenderMode::Rendered, "http", "example", "/"),
        test_5: ("example", GosubRenderMode::Rendered, "https", "example", "/"),

        test_11: ("source:https://example.com", GosubRenderMode::Source, "https", "example.com", "/"),
        test_12: ("raw:example.com", GosubRenderMode::RawSource, "https", "example.com", "/"),
        test_14: ("view-source:http://example", GosubRenderMode::Source, "http", "example", "/"),
        test_15: ("source:example", GosubRenderMode::Source, "https", "example", "/"),

        test_31: ("source:ftp://example.com/foo/bar", GosubRenderMode::Source, "ftp", "example.com", "/foo/bar"),
        test_32: ("raw:gopher://example.com", GosubRenderMode::RawSource, "gopher", "example.com", ""),
        test_34: ("xml:example", GosubRenderMode::Xml, "https", "example", "/"),

        test_40: ("about:blank", GosubRenderMode::About, "about", "", "blank"),
    }

    address_invalid! {
        test_invalid_1: "source:about:blank",
        test_invalid_2: "about:http://example.com",
        test_invalid_3: "about:irc://example.com",
    }
}
