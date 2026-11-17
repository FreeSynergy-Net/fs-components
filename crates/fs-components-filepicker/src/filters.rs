//! Filters of a request translated to what the portal wants.
//!
//! Pattern: Adapter (a pure translation, no portal in it, so it is tested everywhere). Rules: media types are lower-cased and
//! must look like `type/subtype` (else they are left out); patterns are kept as written; duplicates go; a filter with neither
//! a pattern nor a media type is dropped (it would show an empty entry); an empty name falls back to its patterns.

use crate::request::Filter;

/// One filter as the portal takes it: a label, globs and media types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortalFilter {
    /// The label.
    pub label: String,
    /// The glob patterns.
    pub globs: Vec<String>,
    /// The media types.
    pub media_types: Vec<String>,
}

fn media_type_ok(value: &str) -> bool {
    let Some((kind, sub)) = value.split_once('/') else {
        return false;
    };
    let token = |s: &str| {
        !s.is_empty()
            && s.bytes().all(|b| {
                b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.' | b'_' | b'*')
            })
    };
    token(kind) && token(sub)
}

fn unique(values: impl Iterator<Item = String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for v in values {
        if !out.contains(&v) {
            out.push(v);
        }
    }
    out
}

/// The portal filters of `filters`, in order.
pub fn to_portal(filters: &[Filter]) -> Vec<PortalFilter> {
    filters
        .iter()
        .filter_map(|f| {
            let globs = unique(
                f.patterns
                    .iter()
                    .map(|p| p.trim().to_owned())
                    .filter(|p| !p.is_empty()),
            );
            let media_types = unique(
                f.media_types
                    .iter()
                    .map(|m| m.trim().to_ascii_lowercase())
                    .filter(|m| media_type_ok(m)),
            );
            if globs.is_empty() && media_types.is_empty() {
                return None;
            }
            let label = if f.name.trim().is_empty() {
                globs
                    .iter()
                    .chain(&media_types)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                f.name.trim().to_owned()
            };
            Some(PortalFilter {
                label,
                globs,
                media_types,
            })
        })
        .collect()
}
