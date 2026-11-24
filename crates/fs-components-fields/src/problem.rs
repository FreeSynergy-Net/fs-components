//! Why typed parts are not a value.
//!
//! Pattern: Value object — a plain code, compared by value; the sentence is the program's words
//! (ADR 0057), looked up by [`Problem::key`].

use fs_types::values::ValueProblem;

/// Why the parts of a field are not (yet) a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Problem {
    /// The e-mail or phone rule of the contract broke; the code says which.
    Contact(ValueProblem),
    /// Neither a given name nor a surname.
    NoName,
    /// A part is there but another part it needs is not (address without place, date without day).
    Incomplete,
    /// A part that must be digits holds something else.
    NotANumber,
    /// The date does not exist (31 February).
    NoSuchDate,
    /// The typed key is not an entry of the list.
    NotInList,
    /// A keyword is too long or holds a character a keyword may not.
    BadKeyword,
    /// More keywords than a set holds.
    TooMany,
    /// A reference must be absolute: it needs a scheme (`https:`).
    NotAbsolute,
    /// Not an IRI, or too long, or nothing after the scheme.
    BadReference,
    /// A link has a space or a control character.
    LinkCharacter,
    /// A link has no scheme (`example.org`).
    LinkNoScheme,
    /// A link has a scheme that is not allowed (`javascript:`).
    LinkScheme,
    /// A link has a scheme but nothing after it that names where to go.
    LinkNoHost,
    /// A whole number is too large or too small for the type (`whole` below 0, beyond 64 bit).
    OutOfRange,
    /// A yes/no value that is neither yes nor no.
    NotYesNo,
    /// A text has a control character that is not a line break or a tab.
    ControlCharacter,
    /// An hour, minute or second out of its range.
    BadTime,
    /// The offset is not `Z` or `+HH:MM` / `-HH:MM`.
    BadOffset,
    /// Weeks are given together with days or a clock time (a duration has weeks alone).
    WeeksAlone,
    /// The parts are not a duration of the standard (too large).
    BadDuration,
    /// The time zone is not a name of the IANA database.
    UnknownZone,
}

impl Problem {
    /// The translation key of the sentence, stable.
    pub const fn key(self) -> &'static str {
        match self {
            Problem::Contact(ValueProblem::MissingAt) => "problem.email.missing-at",
            Problem::Contact(ValueProblem::ManyAt) => "problem.email.many-at",
            Problem::Contact(ValueProblem::EmptyLocal) => "problem.email.empty-local",
            Problem::Contact(ValueProblem::EmptyDomain) => "problem.email.empty-domain",
            Problem::Contact(ValueProblem::LocalCharacter) => "problem.email.local-character",
            Problem::Contact(ValueProblem::LocalTooLong) => "problem.email.local-too-long",
            Problem::Contact(ValueProblem::BadDomain) => "problem.email.bad-domain",
            Problem::Contact(ValueProblem::DomainWithoutDot) => "problem.email.domain-without-dot",
            Problem::Contact(ValueProblem::AddressTooLong) => "problem.email.too-long",
            Problem::Contact(ValueProblem::NoCountryCode) => "problem.phone.no-country-code",
            Problem::Contact(ValueProblem::PhoneCharacter) => "problem.phone.character",
            Problem::Contact(ValueProblem::PhoneLength) => "problem.phone.length",
            Problem::Contact(ValueProblem::Empty) => "problem.empty",
            // `ValueProblem` is `non_exhaustive` (fs-types 0.0.7): a rule added later says "incomplete" until it gets its own sentence.
            Problem::Contact(_) => "problem.incomplete",
            Problem::NoName => "problem.name.none",
            Problem::Incomplete => "problem.incomplete",
            Problem::NotANumber => "problem.not-a-number",
            Problem::NoSuchDate => "problem.date.no-such-date",
            Problem::NotInList => "problem.choice.not-in-list",
            Problem::BadKeyword => "problem.keywords.bad-word",
            Problem::TooMany => "problem.keywords.too-many",
            Problem::NotAbsolute => "problem.reference.not-absolute",
            Problem::BadReference => "problem.reference.bad",
            Problem::LinkCharacter => "problem.link.character",
            Problem::LinkNoScheme => "problem.link.no-scheme",
            Problem::LinkScheme => "problem.link.scheme",
            Problem::LinkNoHost => "problem.link.no-host",
            Problem::OutOfRange => "problem.number.out-of-range",
            Problem::NotYesNo => "problem.flag.bad",
            Problem::ControlCharacter => "problem.multiline.control-character",
            Problem::BadTime => "problem.date-time.bad-time",
            Problem::BadOffset => "problem.date-time.bad-offset",
            Problem::WeeksAlone => "problem.duration.weeks-alone",
            Problem::BadDuration => "problem.duration.bad",
            Problem::UnknownZone => "problem.zone.unknown",
        }
    }
}
