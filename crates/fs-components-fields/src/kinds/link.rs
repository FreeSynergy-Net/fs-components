//! Link: an address a person can open (RFC 3986 URI with a scheme out of a short list).
//!
//! Pattern: Strategy (one [`BasicField`] implementation). One part, the address. Only `http`, `https`, `mailto` and `tel` are
//! links: another scheme (`javascript:`, `data:`, `file:`) is refused, because a link is opened by activating it and the
//! scheme decides what then runs. The joined form is the trimmed address as it was typed; nothing is rewritten.

use fs_components_styles::Glyph;

use crate::field::{BasicField, Segment, is_blank};
use crate::problem::Problem;

/// The link type.
#[derive(Debug, Clone, Copy)]
pub struct Link;

/// The schemes a link may have.
pub const SCHEMES: [&str; 4] = ["http", "https", "mailto", "tel"];

const SEGMENTS: [Segment; 1] = [Segment {
    key: "url",
    weight: 1,
    required: true,
}];

/// The scheme of `text` in lower case, if it starts with one (`letter *(letter / digit / "+" / "-" / ".") ":"`).
fn scheme(text: &str) -> Option<String> {
    let (head, _) = text.split_once(':')?;
    let mut chars = head.chars();
    let first = chars.next()?;
    (first.is_ascii_alphabetic()
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.')))
    .then(|| head.to_ascii_lowercase())
}

impl BasicField for Link {
    fn key(&self) -> &'static str {
        "link"
    }

    fn glyph(&self) -> Glyph {
        Glyph::Share
    }

    fn segments(&self) -> &'static [Segment] {
        &SEGMENTS
    }

    fn problem(&self, parts: &[String]) -> Option<Problem> {
        if is_blank(parts) {
            return None;
        }
        let text = parts[0].trim();
        if text.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Some(Problem::LinkCharacter);
        }
        let Some(scheme) = scheme(text) else {
            return Some(Problem::LinkNoScheme);
        };
        if !SCHEMES.contains(&scheme.as_str()) {
            return Some(Problem::LinkScheme);
        }
        let rest = &text[scheme.len() + 1..];
        let needs_host = matches!(scheme.as_str(), "http" | "https");
        let host = rest.strip_prefix("//").map(|r| {
            r.split(['/', '?', '#'])
                .next()
                .unwrap_or_default()
                .rsplit('@')
                .next()
                .unwrap_or_default()
        });
        match (needs_host, host) {
            (true, Some(h)) if !h.is_empty() && !h.starts_with(':') => None,
            (true, _) => Some(Problem::LinkNoHost),
            (false, _) if rest.is_empty() => Some(Problem::LinkNoHost),
            (false, _) => None,
        }
    }

    fn split(&self, text: &str) -> Vec<String> {
        vec![text.trim().to_string()]
    }

    fn join(&self, parts: &[String]) -> String {
        parts
            .first()
            .map_or_else(String::new, |p| p.trim().to_string())
    }

    fn compact(&self, parts: &[String]) -> String {
        self.join(parts)
    }

    fn href(&self, parts: &[String]) -> Option<String> {
        let text = self.join(parts);
        (!text.is_empty() && self.problem(parts).is_none()).then_some(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(text: &str) -> Option<Problem> {
        Link.problem(&[text.to_string()])
    }

    #[test]
    fn what_is_a_link() {
        assert_eq!(p(""), None);
        assert_eq!(p("https://example.org/a?b=c#d"), None);
        assert_eq!(p("HTTP://example.org"), None);
        assert_eq!(p("https://user@example.org:8080/"), None);
        assert_eq!(p("mailto:ada@example.org"), None);
        assert_eq!(p("tel:+4930123456"), None);
        assert_eq!(p("example.org"), Some(Problem::LinkNoScheme));
        assert_eq!(p("https//example.org"), Some(Problem::LinkNoScheme));
        assert_eq!(p("https://"), Some(Problem::LinkNoHost));
        assert_eq!(p("https:///path"), Some(Problem::LinkNoHost));
        assert_eq!(p("https://:80/"), Some(Problem::LinkNoHost));
        assert_eq!(p("mailto:"), Some(Problem::LinkNoHost));
        assert_eq!(p("https://exa mple.org"), Some(Problem::LinkCharacter));
    }

    #[test]
    fn a_scheme_that_runs_something_is_no_link() {
        for bad in [
            "javascript:alert(1)",
            "JavaScript:alert(1)",
            "data:text/html,x",
            "file:///etc/passwd",
            "ftp://example.org",
        ] {
            assert_eq!(p(bad), Some(Problem::LinkScheme), "{bad}");
        }
    }

    #[test]
    fn only_a_good_link_has_an_address_to_open() {
        let good = vec!["  https://example.org  ".to_string()];
        assert_eq!(Link.href(&good).as_deref(), Some("https://example.org"));
        assert_eq!(Link.join(&good), "https://example.org");
        assert_eq!(Link.href(&["javascript:x".to_string()]), None);
        assert_eq!(Link.href(&[String::new()]), None);
    }
}
