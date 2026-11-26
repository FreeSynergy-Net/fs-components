//! What a card is and what is carried: a reference to an object, and the place it stands in or goes to.
//!
//! Pattern: Value Object (plain data compared by value). A card has no owner of its own, it only points to the object (ADR 0103 section 6,
//! ADR 0098 E5): the object IRI and the media type, the types of `fs-types`, never a bare text.

use fs_types::{BasicValue, MediaType, Reference, TypeProblem};

/// The reference a card stands for: the IRI of the object (its AS2 `id`) and its media type.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CardRef {
    /// The object.
    pub iri: Reference,
    /// What it is.
    pub media_type: MediaType,
}

impl CardRef {
    /// A reference of `iri` (an absolute IRI) of the media type `media_type` (for example `application/activity+json`).
    ///
    /// # Errors
    /// The IRI is not an absolute IRI: [`TypeProblem`] of `fs-types`.
    pub fn parse(iri: &str, media_type: &str) -> Result<Self, TypeProblem> {
        Ok(Self {
            iri: Reference::parse(iri)?,
            media_type: MediaType(media_type.to_owned()),
        })
    }

    /// The IRI as the program wrote it (what an id of an element is made from).
    pub fn iri_text(&self) -> String {
        self.iri.canonical()
    }
}

/// A place a card stands in or can land: the cell (`lane` x `column`) and the two neighbours there. The names are the arguments of
/// the slot `drop` of the block `board` ([`Target::ARGS`], one place for the schema: the contract; the drop is at the container).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Target {
    /// The lane (the row of the cell); `None` when the container has no lanes.
    pub lane: Option<String>,
    /// The column.
    pub column: String,
    /// The card right above the place; `None` = the top of the cell.
    pub after: Option<CardRef>,
    /// The card right below the place; `None` = the end of the cell.
    pub before: Option<CardRef>,
}

impl Target {
    /// The arguments a drop names, as the contract declares them (`board.drop` in `contract/blocks/widgets.toml`).
    pub const ARGS: [&'static str; 4] = ["column", "lane", "after", "before"];

    /// The cell without neighbours.
    pub fn cell(lane: Option<&str>, column: &str) -> Self {
        Self {
            lane: lane.map(str::to_owned),
            column: column.to_owned(),
            after: None,
            before: None,
        }
    }
}

/// What is carried: the card and where it stood when it was picked up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Carry {
    /// The card.
    pub key: CardRef,
    /// Where it stood.
    pub from: Target,
}
