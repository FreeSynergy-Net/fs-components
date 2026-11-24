//! Date and clock time with an offset: year, month, day, hour, minute, second, offset (RFC 3339 `date-time`).
//!
//! Pattern: Strategy (one [`BasicField`] implementation) over the rule of the contract crate: whether the parts are a value is
//! asked of `fs_types::DateTime::parse` (the standard is the type's, not ours). The stored form keeps the offset the person wrote
//! (`2026-11-10T14:30:00+01:00`); [`utc_text`] gives the same instant normalised to UTC (`...Z`, ADR on times: UTC inside, the
//! zone beside it), for a program that compares or stores instants. A value without offset and zone is a wall-clock time (RFC 8984 `LocalDateTime`): shown as written, never given a `Z`, never converted. The date part follows [`Date`].

use fs_components_styles::Glyph;
use fs_types::{BasicValue, DateTime as Instant, LocalDateTime, Utc};

use super::date::Date;
use super::email::padded;
use crate::field::{BasicField, Segment, is_blank};
use crate::problem::Problem;

/// The date-time type.
#[derive(Debug, Clone, Copy)]
pub struct DateTime;

const fn seg(key: &'static str, weight: u8, required: bool) -> Segment {
    Segment {
        key,
        weight,
        required,
    }
}

const SEGMENTS: [Segment; 7] = [
    seg("year", 3, true),
    seg("month", 2, true),
    seg("day", 2, true),
    seg("hour", 2, true),
    seg("minute", 2, true),
    seg("second", 2, false),
    seg("offset", 3, false),
];

fn digits_ok(text: &str, max: usize) -> bool {
    let t = text.trim();
    t.is_empty() || (t.len() <= max && t.chars().all(|c| c.is_ascii_digit()))
}

fn small(text: &str) -> u32 {
    text.trim().parse().unwrap_or(0)
}

fn offset_word(text: &str) -> String {
    let t = text.trim();
    if t.eq_ignore_ascii_case("z") {
        "Z".to_owned()
    } else {
        t.to_owned()
    }
}

/// Whether the text is an instant (with an offset) or a wall-clock time (no zone, no offset: RFC 8984 `LocalDateTime`).
fn reads(text: &str) -> bool {
    Instant::parse(text).is_ok() || LocalDateTime::parse(text).is_ok()
}

/// The instant normalised to UTC (`YYYY-MM-DDTHH:MM:SS.mmmZ`); `None` while the parts are not a value.
pub fn utc_text(parts: &[String]) -> Option<String> {
    let text = DateTime.join(parts);
    Utc::parse(&text).ok()?.to_rfc3339()
}

impl BasicField for DateTime {
    fn key(&self) -> &'static str {
        "dateTime"
    }

    fn glyph(&self) -> Glyph {
        Glyph::Calendar
    }

    fn segments(&self) -> &'static [Segment] {
        &SEGMENTS
    }

    fn joiner(&self, after: usize) -> Option<&'static str> {
        match after {
            0 | 1 => Some("-"),
            2 => Some("T"),
            3 | 4 => Some(":"),
            _ => None,
        }
    }

    fn settle(&self, parts: &[String]) -> Vec<String> {
        let parts = padded(parts, 7);
        let rest_blank = parts[1..].iter().all(|p| p.trim().is_empty());
        if rest_blank && !parts[0].trim().is_empty() {
            let spread = self.split(&parts[0]);
            if spread.iter().skip(1).any(|p| !p.is_empty()) {
                return spread;
            }
        }
        parts
    }

    fn problem(&self, parts: &[String]) -> Option<Problem> {
        let parts = padded(parts, 7);
        if is_blank(&parts) {
            return None;
        }
        if !digits_ok(&parts[3], 2) || !digits_ok(&parts[4], 2) || !digits_ok(&parts[5], 2) {
            return Some(Problem::NotANumber);
        }
        let required = [0usize, 1, 2, 3, 4];
        if required.iter().any(|&i| parts[i].trim().is_empty()) {
            // the date alone has its own sentences (a letter in the month, 31 February)
            if let Some(p) = Date.problem(&parts[..3]) {
                return Some(p);
            }
            return Some(Problem::Incomplete);
        }
        if let Some(p) = Date.problem(&parts[..3]) {
            return Some(p);
        }
        if small(&parts[3]) > 23 || small(&parts[4]) > 59 || small(&parts[5]) > 59 {
            return Some(Problem::BadTime);
        }
        let joined = self.join(&parts);
        if parts[6].trim().is_empty() {
            // no zone, no offset: a wall-clock time (RFC 8984 LocalDateTime), valid as it is
            return LocalDateTime::parse(&joined)
                .is_err()
                .then_some(Problem::BadTime);
        }
        Instant::parse(&joined)
            .is_err()
            .then_some(Problem::BadOffset)
    }

    fn split(&self, text: &str) -> Vec<String> {
        let canon = match (Instant::parse(text), LocalDateTime::parse(text)) {
            (Ok(parsed), _) => parsed.canonical(),
            (_, Ok(local)) => local.canonical(),
            _ => return vec![text.trim().to_owned()],
        };
        let (date, rest) = canon.split_once('T').unwrap_or((&canon, ""));
        let d: Vec<&str> = date.split('-').collect();
        let cut = rest.find(['Z', '+', '-']).unwrap_or(rest.len());
        let (clock, offset) = rest.split_at(cut);
        let c: Vec<&str> = clock.split(':').collect();
        let second = c.get(2).and_then(|s| s.split('.').next()).unwrap_or("");
        let at = |v: &[&str], i: usize| v.get(i).copied().unwrap_or("").to_owned();
        vec![
            at(&d, 0),
            at(&d, 1),
            at(&d, 2),
            at(&c, 0),
            at(&c, 1),
            second.to_owned(),
            offset.to_owned(),
        ]
    }

    fn join(&self, parts: &[String]) -> String {
        let parts = padded(parts, 7);
        if is_blank(&parts) {
            return String::new();
        }
        let t = |i: usize| parts[i].trim();
        let second = if t(5).is_empty() { "00" } else { t(5) };
        format!(
            "{:0>4}-{:0>2}-{:0>2}T{:0>2}:{:0>2}:{second:0>2}{}",
            t(0),
            t(1),
            t(2),
            t(3),
            t(4),
            offset_word(&parts[6])
        )
    }

    /// Only a completely read instant is shown in its composed form; anything else stays as it came, raw and never padded.
    fn compact(&self, parts: &[String]) -> String {
        let joined = self.join(parts);
        if joined.is_empty() || (self.problem(parts).is_none() && reads(&joined)) {
            return joined;
        }
        parts
            .iter()
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(v: [&str; 7]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn rules() {
        assert_eq!(DateTime.problem(&p([""; 7])), None);
        let ok = p(["2026", "11", "10", "14", "30", "", "+01:00"]);
        assert_eq!(DateTime.problem(&ok), None);
        let mut v = ok.clone();
        v[6] = String::new();
        assert_eq!(DateTime.problem(&v), None, "no offset: a wall-clock time");
        v[0] = String::new();
        assert_eq!(DateTime.problem(&v), Some(Problem::Incomplete));
        let mut v = ok.clone();
        v[3] = "24".into();
        assert_eq!(DateTime.problem(&v), Some(Problem::BadTime));
        let mut v = ok.clone();
        v[4] = "60".into();
        assert_eq!(DateTime.problem(&v), Some(Problem::BadTime));
        let mut v = ok.clone();
        v[5] = "60".into();
        assert_eq!(DateTime.problem(&v), Some(Problem::BadTime));
        v[5] = "59".into();
        assert_eq!(DateTime.problem(&v), None);
        let mut v = ok.clone();
        v[3] = "1x".into();
        assert_eq!(DateTime.problem(&v), Some(Problem::NotANumber));
        let mut v = ok.clone();
        v[1] = "02".into();
        v[2] = "30".into();
        assert_eq!(DateTime.problem(&v), Some(Problem::NoSuchDate));
        let mut v = ok;
        v[6] = "+25:00".into();
        assert_eq!(DateTime.problem(&v), Some(Problem::BadOffset));
    }

    #[test]
    fn the_signs_between_the_parts_are_those_of_rfc_3339() {
        let signs: Vec<_> = (0..7).map(|i| DateTime.joiner(i)).collect();
        assert_eq!(
            signs,
            [
                Some("-"),
                Some("-"),
                Some("T"),
                Some(":"),
                Some(":"),
                None,
                None
            ]
        );
    }

    #[test]
    fn joined_split_settle_and_utc() {
        let v = p(["2026", "3", "9", "7", "5", "", "z"]);
        assert_eq!(DateTime.join(&v), "2026-03-09T07:05:00Z");
        assert_eq!(DateTime.join(&p([""; 7])), "");
        let back = DateTime.split("2026-11-10T14:30:00+01:00");
        assert_eq!(back, p(["2026", "11", "10", "14", "30", "00", "+01:00"]));
        assert_eq!(DateTime.join(&back), "2026-11-10T14:30:00+01:00");
        let mut pasted = p([""; 7]);
        pasted[0] = "2026-11-10T14:30:00Z".into();
        assert_eq!(
            DateTime.settle(&pasted),
            p(["2026", "11", "10", "14", "30", "00", "Z"])
        );
        assert_eq!(utc_text(&back).as_deref(), Some("2026-11-10T13:30:00.000Z"));
        assert_eq!(utc_text(&p(["2026", "", "", "", "", "", ""])), None);
    }

    /// Mutant: `compact` is `join` again (pads) -> `an_unreadable_instant_is_shown_raw_never_padded` fails.
    #[test]
    fn an_unreadable_instant_is_shown_raw_never_padded() {
        let raw = "2026-11-26T12:00:00"; // no offset: not RFC 3339
        assert_eq!(DateTime.compact(&DateTime.split(raw)), raw);
        assert_eq!(DateTime.compact(&DateTime.split("tomorrow")), "tomorrow");
        let read = "2026-11-26T12:00:00+01:00";
        assert_eq!(DateTime.compact(&DateTime.split(read)), read);
        assert_eq!(DateTime.compact(&DateTime.split("")), "");
    }

    /// Partial values stay as they came (today's behaviour, documented here): date only, time only, date with a time but no offset.
    #[test]
    fn partial_instants_stay_raw() {
        for raw in ["2026-11-26", "12:00:00", "2026-11-26T12:00", "2026-11-26T"] {
            assert_eq!(DateTime.compact(&DateTime.split(raw)), raw, "{raw}");
        }
    }
}
