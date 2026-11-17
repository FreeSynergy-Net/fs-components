//! `file://` URIs of the portal turned into paths.
//!
//! Pattern: Adapter (pure functions, no portal). The portal answers with URIs; only the `file` scheme with an empty or
//! `localhost` host is a local path. Percent escapes are decoded as bytes (a path need not be UTF-8 text on Linux, but the
//! portal sends UTF-8). Anything else is refused, not guessed.

use std::path::PathBuf;

fn hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// The local path of `uri`, or `None` when it is no `file:` URI of this machine or has a broken escape.
pub fn path_of_uri(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let (host, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => return None,
    };
    if !(host.is_empty() || host.eq_ignore_ascii_case("localhost")) {
        return None;
    }
    let path = path.split(['?', '#']).next().unwrap_or(path);
    let bytes = path.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let (h, l) = (hex(*bytes.get(i + 1)?)?, hex(*bytes.get(i + 2)?)?);
            out.push(h * 16 + l);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    let text = String::from_utf8(out).ok()?;
    Some(PathBuf::from(text))
}
