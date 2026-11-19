//! Duration: a span of time as number and unit pairs (RFC 8984 section 1.4.6, the JSCalendar subset of ISO 8601 durations).
//!
//! Pattern: Strategy (one [`BasicField`] implementation) over the rule of the contract crate: whether the parts are a value is asked
//! of `fs_types::Duration::parse`. Each part is a whole number with its unit as the label (weeks, days, hours, minutes, seconds);
//! the units are kept as written, never converted (`60` minutes stays `PT60M`). Weeks stand alone (RFC 8984: no weeks with days).

use fs_components_styles::Glyph;
use fs_types::{BasicValue, Duration as Span};

use super::email::padded;
use crate::field::{BasicField, Segment, is_blank};
use crate::problem::Problem;

/// The duration type.
#[derive(Debug, Clone, Copy)]
pub struct Duration;

const fn seg(key: &'static str) -> Segment {
    Segment {
        key,
        weight: 2,
        required: false,
    }
}

const SEGMENTS: [Segment; 5] = [
    seg("weeks"),
    seg("days"),
    seg("hours"),
    seg("minutes"),
    seg("seconds"),
];

fn num(text: &str) -> Option<&str> {
    let t = text.trim();
    (!t.is_empty())
        .then_some(t.trim_start_matches('0'))
        .map(|s| if s.is_empty() { "0" } else { s })
}

impl BasicField for Duration {
    fn key(&self) -> &'static str {
        "duration"
    }

    fn glyph(&self) -> Glyph {
        Glyph::History
    }

    fn segments(&self) -> &'static [Segment] {
        &SEGMENTS
    }

    fn settle(&self, parts: &[String]) -> Vec<String> {
        let parts = padded(parts, 5);
        let first = parts[0].trim();
        if first.len() > 1
            && first.starts_with(['P', 'p'])
            && parts[1..].iter().all(|p| p.trim().is_empty())
        {
            let spread = self.split(first);
            if spread.iter().any(|p| !p.is_empty()) {
                return spread;
            }
        }
        parts
    }

    fn problem(&self, parts: &[String]) -> Option<Problem> {
        let parts = padded(parts, 5);
        if is_blank(&parts) {
            return None;
        }
        let digits = |p: &String| p.trim().chars().all(|c| c.is_ascii_digit());
        if !parts.iter().all(digits) || parts.iter().any(|p| p.trim().len() > 15) {
            return Some(Problem::NotANumber);
        }
        if !parts[0].trim().is_empty() && parts[1..].iter().any(|p| !p.trim().is_empty()) {
            return Some(Problem::WeeksAlone);
        }
        Span::parse(&self.join(&parts))
            .is_err()
            .then_some(Problem::BadDuration)
    }

    fn split(&self, text: &str) -> Vec<String> {
        let t = text.trim().to_ascii_uppercase();
        let mut out = vec![String::new(); 5];
        let Some(rest) = t.strip_prefix('P') else {
            out[0] = text.trim().to_owned();
            return out;
        };
        let mut time = false;
        let mut number = String::new();
        for c in rest.chars() {
            match c {
                'T' => time = true,
                '0'..='9' => number.push(c),
                unit => {
                    let slot = match (unit, time) {
                        ('W', false) => 0,
                        ('D', false) => 1,
                        ('H', true) => 2,
                        ('M', true) => 3,
                        ('S', true) => 4,
                        _ => {
                            return vec![
                                text.trim().to_owned(),
                                String::new(),
                                String::new(),
                                String::new(),
                                String::new(),
                            ];
                        }
                    };
                    out[slot] = std::mem::take(&mut number);
                }
            }
        }
        out
    }

    fn join(&self, parts: &[String]) -> String {
        let parts = padded(parts, 5);
        if is_blank(&parts) {
            return String::new();
        }
        let n = |i: usize| num(&parts[i]);
        if let Some(w) = n(0) {
            return format!("P{w}W");
        }
        let mut text = String::from("P");
        if let Some(d) = n(1) {
            text.push_str(&format!("{d}D"));
        }
        let (h, m, s) = (n(2), n(3), n(4));
        if h.is_some() || m.is_some() || s.is_some() {
            text.push('T');
            if let Some(h) = h {
                text.push_str(&format!("{h}H"));
            }
            match (m, h.is_some() && s.is_some()) {
                (Some(m), _) => text.push_str(&format!("{m}M")),
                (None, true) => text.push_str("0M"),
                (None, false) => {}
            }
            if let Some(s) = s {
                text.push_str(&format!("{s}S"));
            }
        }
        text
    }

    fn compact(&self, parts: &[String]) -> String {
        self.join(parts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(v: [&str; 5]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn rules() {
        assert_eq!(Duration.problem(&p([""; 5])), None);
        assert_eq!(Duration.problem(&p(["", "1", "2", "30", ""])), None);
        assert_eq!(Duration.problem(&p(["2", "", "", "", ""])), None);
        assert_eq!(
            Duration.problem(&p(["2", "1", "", "", ""])),
            Some(Problem::WeeksAlone)
        );
        assert_eq!(
            Duration.problem(&p(["", "x", "", "", ""])),
            Some(Problem::NotANumber)
        );
        assert_eq!(
            Duration.problem(&p(["", "", "", "", "99999999999999999999"])),
            Some(Problem::NotANumber)
        );
        assert_eq!(
            Duration.problem(&p(["", "", "", "", "9999999999999999"])),
            Some(Problem::NotANumber)
        );
    }

    #[test]
    fn joined_split_settle() {
        assert_eq!(Duration.join(&p(["", "1", "2", "30", ""])), "P1DT2H30M");
        assert_eq!(Duration.join(&p(["", "", "1", "", "5"])), "PT1H0M5S");
        assert_eq!(Duration.join(&p(["3", "", "", "", ""])), "P3W");
        assert_eq!(Duration.join(&p(["", "", "", "007", ""])), "PT7M");
        assert_eq!(Duration.join(&p([""; 5])), "");
        assert_eq!(Duration.split("P1DT2H30M"), p(["", "1", "2", "30", ""]));
        assert_eq!(Duration.split("PT45S"), p(["", "", "", "", "45"]));
        assert_eq!(
            Duration.settle(&p(["P1DT2H", "", "", "", ""])),
            p(["", "1", "2", "", ""])
        );
        // what is written is read again by the type
        let v = p(["", "1", "2", "", "5"]);
        assert!(Span::parse(&Duration.join(&v)).is_ok());
    }
}
