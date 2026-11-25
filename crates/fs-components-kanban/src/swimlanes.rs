//! The lanes of a board as a model (no window): the heads of the lanes (a second source, the form `rows`) and the cards, grouped by
//! one field ([`lanes_of`]).
//!
//! The rule (draft lanes, section 1b): the heads come first, in their own order; a card value no head names gets a lane at the end
//! (nothing is hidden); a head without a card is an empty lane; a card with several values stands in several lanes. A card without
//! a value for the field stands in no lane (the board still draws it in its column).
//!
//! Pattern: Value object + pure function – plain data in (heads, cards), plain data out (lanes); no source, no window.

/// The head of one lane: the value cards name it by, and the label the reader sees.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneHead {
    /// The value a card's field carries to belong to this lane.
    pub key: String,
    /// The name of the lane as the reader should see it.
    pub label: String,
}

impl LaneHead {
    /// A head.
    pub fn new(key: &str, label: &str) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
        }
    }
}

/// A card as far as lanes care: its key and the values of its fields (a field may carry several values).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneCard {
    /// The card key.
    pub key: String,
    fields: Vec<(String, Vec<String>)>,
}

impl LaneCard {
    /// A card without field values.
    pub fn new(key: &str) -> Self {
        Self {
            key: key.into(),
            fields: Vec::new(),
        }
    }

    /// With the values of one field (set twice: the values are added).
    #[must_use]
    pub fn with(mut self, field: &str, values: &[&str]) -> Self {
        let own: Vec<String> = values.iter().map(|v| (*v).to_owned()).collect();
        match self.fields.iter_mut().find(|(name, _)| name == field) {
            Some((_, have)) => have.extend(own),
            None => self.fields.push((field.into(), own)),
        }
        self
    }

    /// The values of a field (empty = none).
    pub fn values(&self, field: &str) -> &[String] {
        self.fields
            .iter()
            .find(|(name, _)| name == field)
            .map_or(&[][..], |(_, v)| &v[..])
    }
}

/// One lane: its key, its label, the keys of its cards in the order they were handed in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lane {
    /// The lane key (the value the cards carry).
    pub key: String,
    /// The name shown; a lane without a head shows its key.
    pub label: String,
    /// Whether a head named this lane (`false` = it exists only because cards name it).
    pub headed: bool,
    /// The cards of the lane.
    pub cards: Vec<String>,
}

/// The lanes of `cards` grouped by the field `grouping`, with the `heads` in their order. A head named twice counts once (the first).
/// Blank values (only spaces) are no value.
#[must_use]
pub fn lanes_of(heads: &[LaneHead], cards: &[LaneCard], grouping: &str) -> Vec<Lane> {
    let mut lanes: Vec<Lane> = Vec::new();
    for head in heads {
        if lanes.iter().all(|l| l.key != head.key) {
            lanes.push(Lane {
                key: head.key.clone(),
                label: head.label.clone(),
                headed: true,
                cards: Vec::new(),
            });
        }
    }
    for card in cards {
        for value in card.values(grouping) {
            let value = value.trim();
            if value.is_empty() {
                continue;
            }
            let at = lanes
                .iter()
                .position(|l| l.key == value)
                .unwrap_or_else(|| {
                    lanes.push(Lane {
                        key: value.into(),
                        label: value.into(),
                        headed: false,
                        cards: Vec::new(),
                    });
                    lanes.len() - 1
                });
            if !lanes[at].cards.contains(&card.key) {
                lanes[at].cards.push(card.key.clone());
            }
        }
    }
    lanes
}

/// What the host says about lanes ([`BoardSource::lane_plan`](crate::BoardSource::lane_plan)): the board takes the state as an input and
/// guesses nothing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum LanePlan {
    /// No lanes asked for: the board draws the columns as before.
    #[default]
    Off,
    /// Lanes were asked for (a source of heads is declared) but the heads did not come: the columns are drawn as before, with a line
    /// that says the list of lanes is missing (never a silent plain board).
    Missing,
    /// Lanes: the cards grouped by the field `grouping` (today only `assignee`), the heads in their own order.
    On {
        /// The field of the card the lanes follow.
        grouping: String,
        /// The heads (names are text of the host; the key is only the value cards carry).
        heads: Vec<LaneHead>,
    },
}

impl LaneCard {
    /// The card of the board as far as lanes care: the values of the field `grouping` it carries (`assignee` = its assignees).
    pub fn of_card(card: &crate::Card, grouping: &str) -> Self {
        let own = Self::new(&card.key);
        if grouping == "assignee" {
            let names: Vec<&str> = card.assignees.iter().map(String::as_str).collect();
            own.with(grouping, &names)
        } else {
            own
        }
    }
}
