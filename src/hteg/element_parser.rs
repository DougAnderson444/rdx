//! Parses and ElementRef into HtmlElement
#![allow(dead_code)]

use super::types::HtmlElement;
use crate::Error;
use ahash::AHashMap;

use html5ever::tendril::TendrilSink;
use html5ever::{local_name, ns, parse_fragment, QualName};
use markup5ever::namespace_url;
use markup5ever_rcdom::RcDom;

/// A Parser that will parse an html string into a Vec<HtmlElement>
/// Holds a cache to avoid parsing the same html multiple times.
#[derive(Default, Clone)]
pub struct Parser {
    cache: AHashMap<String, HtmlElement>,
}

impl Parser {
    /// Parse an html string into a Vec<HtmlElement>
    pub fn parse(&mut self, html: &str) -> Result<HtmlElement, Error> {
        // Before usign scraper, we check the cache to see if we have already parsed this html
        // and if so, we return the cached value. This is a simple way to avoid parsing the same
        // html multiple times, which eats memory and cpu.
        if let Some(elements) = self.cache.get(&html.to_string()) {
            return Ok(elements.clone());
        }

        tracing::info!("No cache found. Parsing html: {}", html);

        let elements = parse(html)?;
        // Map this html to Vec<HtmlElement> in a cache so that we can return
        // any subsequent calls to this same html without having to reparse it.
        self.cache.insert(html.to_string(), elements.clone());

        Ok(elements)
    }
}

pub(crate) fn parse(html: &str) -> Result<HtmlElement, Error> {
    // Parse the HTML
    let dom = parse_fragment(
        RcDom::default(),
        Default::default(),
        QualName::new(None, ns!(), local_name!("div")), // Context element
        vec![],
    )
    .from_utf8()
    .read_from(&mut html.as_bytes())
    .unwrap();

    let ast = HtmlElement::from_node(&dom.document).unwrap();
    Ok(ast)
}

#[cfg(test)]
mod tests {}
