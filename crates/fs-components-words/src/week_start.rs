//! `with_week_start`: the person's choice of the first weekday as a BCP 47 extension of the locale.
//!
//! Pattern: Decorator (pure function) – returns the same locale with the Unicode extension keyword `fw` set (`de` and `sun`
//! give `de-u-fw-sun`), which `DateNames::first_weekday` of the implementation honours. The setting is
//! `fs.shell.week-start` = `auto|mon|tue|wed|thu|fri|sat|sun`; `auto` removes the keyword, so the CLDR week of the region
//! decides. Other keywords of the extension (`-u-ca-…`, `-u-nu-…`) are kept, in the order BCP 47 asks for (by key).

use crate::locale::Locale;

/// The values the setting `fs.shell.week-start` may hold: `auto` first, then the weekdays from Monday.
pub const WEEK_START_CHOICES: [&str; 8] = ["auto", "mon", "tue", "wed", "thu", "fri", "sat", "sun"];

/// `locale` with the first weekday `choice` (`mon`…`sun`); `auto` or any other value leaves the week to the region
/// (an existing `fw` keyword is removed). Never fails: a tag that cannot be rebuilt comes back unchanged.
pub fn with_week_start(locale: &Locale, choice: &str) -> Locale {
    let choice = choice.to_ascii_lowercase();
    let set = WEEK_START_CHOICES[1..].contains(&choice.as_str());
    let tokens: Vec<&str> = locale.as_str().split('-').collect();
    // Where the unicode extension is (or goes): `-u-` if present, else before the first singleton behind `u` (`-x-…`).
    let singleton = |t: &&str| t.len() == 1;
    let (head, tail): (&[&str], &[&str]) = if let Some(at) = tokens.iter().position(|t| *t == "u") {
        (&tokens[..at], &tokens[at + 1..])
    } else {
        let at = tokens
            .iter()
            .position(|t| singleton(t) && *t > "u")
            .unwrap_or(tokens.len());
        (&tokens[..at], &tokens[at..])
    };
    let has_u = head.len() < tokens.len() && tokens[head.len()] == "u";
    // The unicode extension runs to the next singleton; what follows is kept as it is.
    let end = if has_u {
        tail.iter().position(singleton).unwrap_or(tail.len())
    } else {
        0
    };
    let (ext, after) = if has_u {
        tail.split_at(end)
    } else {
        (&tail[..0], tail)
    };
    let first_key = ext.iter().position(|p| p.len() == 2).unwrap_or(ext.len());
    let (attributes, keywords) = ext.split_at(first_key);
    // Keywords: a two-letter key with the values (longer subtags) behind it.
    let mut list: Vec<Vec<&str>> = Vec::new();
    for part in keywords {
        if part.len() == 2 {
            list.push(vec![part]);
        } else if let Some(last) = list.last_mut() {
            last.push(part);
        }
    }
    list.retain(|k| k[0] != "fw");
    if set {
        list.push(vec!["fw", choice.as_str()]);
    }
    list.sort_by(|a, b| a[0].cmp(b[0]));
    let mut out: Vec<&str> = head.to_vec();
    if !attributes.is_empty() || !list.is_empty() {
        out.push("u");
        out.extend(
            attributes
                .iter()
                .copied()
                .chain(list.iter().flatten().copied()),
        );
    }
    out.extend_from_slice(after);
    Locale::parse(&out.join("-")).unwrap_or_else(|_| locale.clone())
}
