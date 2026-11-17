//! P1 probe: pure text rules of the three-part address field (no Blinc): taking a pasted address apart.
//!
//! Tests: covered by `festzeichen.rs` (K cases E2, E5, E6).

/// What the field did to the pasted text, shown to the person (nothing is changed silently).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// `Ada <ada@x.org>` or `mailto:` or blanks were removed.
    Cover,
    /// Two `@`: everything went into the local part.
    ManyAt,
}

/// The three parts and the changes made on the way.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Parts {
    pub local: String,
    pub server: String,
    pub tld: String,
    pub shown: Vec<Change>,
}

/// How the last dot is found (the mutant takes the FIRST one).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DotAt {
    Last,
    First,
}

/// Strip `Name <...>`, `mailto:` and blanks; report whether something was removed.
pub fn strip_cover(text: &str) -> (String, bool) {
    let mut s = text.trim();
    let mut cut = s.len() != text.len();
    if let (Some(a), Some(b)) = (s.rfind('<'), s.rfind('>'))
        && a < b
    {
        s = s[a + 1..b].trim();
        cut = true;
    }
    if s.len() >= 7 && s[..7].eq_ignore_ascii_case("mailto:") {
        s = &s[7..];
        cut = true;
    }
    (s.to_string(), cut)
}

/// Spread a whole address over local, server and tld. `None` if there is no `@` (then it is plain text for one part).
pub fn split_address(text: &str, dot: DotAt) -> Option<Parts> {
    let (s, cut) = strip_cover(text);
    if !s.contains('@') {
        return None;
    }
    let mut shown = Vec::new();
    if cut {
        shown.push(Change::Cover);
    }
    if s.matches('@').count() > 1 {
        shown.push(Change::ManyAt);
        return Some(Parts {
            local: s,
            shown,
            ..Parts::default()
        });
    }
    let (local, rest) = s.split_once('@')?;
    let at = match dot {
        DotAt::Last => rest.rfind('.'),
        DotAt::First => rest.find('.'),
    };
    let (server, tld) = match at {
        Some(i) => (&rest[..i], &rest[i + 1..]),
        None => (rest, ""),
    };
    Some(Parts {
        local: local.to_string(),
        server: server.to_string(),
        tld: tld.to_string(),
        shown,
    })
}
