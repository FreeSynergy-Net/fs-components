//! `Mf2Texts`: the messages of one locale and its fallback chain, as `Words`.
//!
//! Pattern: Adapter + Chain of Responsibility — the first language of the chain (`de-AT`, `de`, then English) that has the
//! message and can fill it wins; nothing found is `Said::missing(id)`. The messages of a language come from a `Messages`
//! store (Strategy, `backend.rs`): packed catalogs (`Mf2Texts::from_packed`, `packed.rs`, the runtime way) or `.mf2` sources
//! compiled on first use (`Mf2Texts::new`, feature `compile`, the old way). Numbers and plural forms come from the CLDR data
//! inside `mf2`. Dates and times are handed over as real `mf2` dates; a language with a date table formats them by the
//! language, one without shows them neutrally and the message stays (see `date_data.rs`).

use fs_components_words::{Args, Said, TextId, Value, Words};
use mf2::{Arg, BidiStrategy, DateTime, FormatContext, Formatter, MsgId, Registry};
use mf2_catalog::Catalog;

use crate::backend::Messages;
use crate::date;
use crate::date_data::registry_for;

/// No isolation marks (U+2066 to U+2069) around inserted values: they are for mixed left-to-right and right-to-left text, and
/// only show as boxes in a font that lacks them. Right to left is not supported yet (DESIGN); a language that needs it
/// switches this on.
static CONTEXT: FormatContext = {
    let mut cx = FormatContext::new(&mf2::host_std::HOST);
    cx.bidi = BidiStrategy::None;
    cx
};

pub(crate) struct Language {
    pub(crate) tag: String,
    pub(crate) messages: Box<dyn Messages>,
    /// The functions of this language: the date functions depend on its date table.
    pub(crate) registry: &'static Registry,
}

/// The texts of one locale.
pub struct Mf2Texts {
    pub(crate) chain: Vec<Language>,
}

impl Language {
    pub(crate) fn say(&self, id: &str, args: &Args) -> Option<String> {
        self.messages.with(id, &mut |catalog, msg, names| {
            // First try: dates as real dates (what `:datetime`, `:date`, `:time` need). A message that uses a date without
            // such a function (`{$day}`) cannot take them; it gets the neutral text instead. The message is never dropped
            // because of a date.
            let dates: Vec<Option<DateTime<'_>>> =
                names.iter().map(|n| datetime_of(args.get(n))).collect();
            let values: Vec<Arg<'_>> = names
                .iter()
                .zip(&dates)
                .map(|(n, d)| {
                    d.as_ref()
                        .map_or_else(|| arg_of(args.get(n), ""), Arg::from)
                })
                .collect();
            let neutral = registry_for(false);
            Self::render(catalog, msg, &values, self.registry)
                // Second try: the date data of this language is damaged inside (a valid table with a bad blob fails only when
                // it formats): the neutral functions take the same real dates and need no data.
                .or_else(|| {
                    (!std::ptr::eq(self.registry, neutral))
                        .then(|| Self::render(catalog, msg, &values, neutral))
                        .flatten()
                })
                .or_else(|| {
                    let owned: Vec<String> = names.iter().map(|n| text_of(args.get(n))).collect();
                    let values: Vec<Arg<'_>> = names
                        .iter()
                        .zip(&owned)
                        .map(|(n, text)| arg_of(args.get(n), text))
                        .collect();
                    Self::render(catalog, msg, &values, neutral)
                })
        })
    }

    fn render(
        catalog: &Catalog,
        msg: MsgId,
        values: &[Arg<'_>],
        registry: &Registry,
    ) -> Option<String> {
        let (mut out, mut errors) = (String::new(), Vec::new());
        Formatter::new(catalog, registry, &CONTEXT).write(msg, values, &mut out, &mut errors);
        (errors.is_empty() && !out.trim().is_empty()).then_some(out)
    }
}

/// A date or time as an `mf2` date (UTC); other values are no dates.
fn datetime_of(value: Option<&Value>) -> Option<DateTime<'static>> {
    match value {
        Some(Value::Date(t) | Value::Time(t)) => DateTime::from_epoch_ms(t.checked_mul(1000)?),
        _ => None,
    }
}

/// The neutral text form of a date or time (only those two need one).
fn text_of(value: Option<&Value>) -> String {
    match value {
        Some(Value::Date(d)) => date::day(*d),
        Some(Value::Time(t)) => date::clock(*t),
        _ => String::new(),
    }
}

fn arg_of<'a>(value: Option<&'a Value>, rendered: &'a str) -> Arg<'a> {
    match value {
        Some(Value::Text(t)) => Arg::Str(t),
        Some(Value::Count(n)) => i64::try_from(*n).map_or(Arg::Unset, Arg::Int),
        Some(Value::Number(n)) => Arg::Float(*n),
        Some(Value::Date(_) | Value::Time(_)) => Arg::Str(rendered),
        None => Arg::Unset,
    }
}

impl Mf2Texts {
    /// Shorthand: the sentence for `id` without values.
    pub fn plain(&self, id: &str) -> String {
        self.say(&TextId::new(id), &Args::new()).text().to_owned()
    }
}

impl Words for Mf2Texts {
    fn say(&self, id: &TextId, args: &Args) -> Said {
        self.chain
            .iter()
            .find_map(|l| l.say(id.as_str(), args))
            .map_or_else(|| Said::missing(id), |text| Said::found(id, text))
    }
}
