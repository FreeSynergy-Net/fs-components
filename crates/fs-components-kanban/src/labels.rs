//! The words of the board workbench: German and English from `MessageFormat` 2 files (ADR 0057), English by default.
//!
//! Pattern: Factory method (`Localized::for_locale`) over embedded files. The program's own sentences (reasons, questions, titles)
//! are its data and do not come from here.

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Args, Locale, Localized, TextId, Words};

/// The two language files; `fs_components_mf2::check_pairs(TEXTS)` must be empty (a test asserts it).
pub const TEXTS: &[Source] = &[
    Source {
        language: "en",
        text: include_str!("texts/board-en.mf2"),
    },
    Source {
        language: "de",
        text: include_str!("texts/board-de.mf2"),
    },
];

/// The words of the board.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// "Add card" (the foot of a column).
    pub add: String,
    /// Placeholder of the quick entry.
    pub quick_hint: String,
    /// A column without cards.
    pub empty: String,
    /// "Moved" (the line under the board).
    pub moved: String,
    /// "Undo".
    pub undo: String,
    /// "Open".
    pub open: String,
    /// "Close" (the frame at the side).
    pub close: String,
    /// Name of the column menu button (assistive technology).
    pub column_menu: String,
    /// "Columns" (the table).
    pub settings: String,
    /// "Move anyway" (answer to the program's question).
    pub yes: String,
    /// "Cancel".
    pub no: String,
    /// "Create" (answer to "N cards?").
    pub create: String,
    /// "set" (an empty property in the frame at the side).
    pub detail_set: String,
    /// "over the limit" (name of the warning symbol).
    pub wip_over: String,
    /// The words of a control that is not finished yet or cannot be used now (the tooltip; never a line of its own).
    pub why: fs_components_widgets::disabled::Labels,
    /// "No column before": the reason of the left move button on the first column.
    pub edge_prev: String,
    /// "No column after": the reason of the right move button on the last column.
    pub edge_next: String,
    /// "Move to..." (the frame at the side).
    pub move_to: String,
    /// What a press on a card says when moving is not supported yet.
    pub move_soon: String,
    /// The lane of cards that stand in no lane.
    pub lane_none: String,
    /// The line that says lanes were asked for but their list is missing.
    pub lanes_missing: String,
    /// The language the words were made for.
    pub locale: Locale,
}

impl Localized for Labels {
    fn for_locale(locale: &Locale) -> Self {
        let t = Mf2Texts::new(TEXTS, locale);
        Self {
            add: t.plain("board-add"),
            quick_hint: t.plain("board-quick-hint"),
            empty: t.plain("board-empty"),
            moved: t.plain("board-moved"),
            undo: t.plain("board-undo"),
            open: t.plain("board-open"),
            close: t.plain("board-close"),
            column_menu: t.plain("board-column-menu"),
            settings: t.plain("board-settings"),
            yes: t.plain("board-yes"),
            no: t.plain("board-no"),
            create: t.plain("board-create"),
            detail_set: t.plain("board-detail-set"),
            wip_over: t.plain("board-wip-over"),
            why: fs_components_widgets::disabled::Labels::for_locale(locale),
            edge_prev: t.plain("board-move-edge-prev"),
            edge_next: t.plain("board-move-edge-next"),
            move_to: t.plain("board-move-to"),
            move_soon: t.plain("board-move-soon"),
            lane_none: t.plain("board-lane-none"),
            lanes_missing: t.plain("board-lanes-missing"),
            locale: locale.clone(),
        }
    }
}

impl Default for Labels {
    fn default() -> Self {
        Self::for_locale(&Locale::english())
    }
}

impl Labels {
    fn say(&self, id: &str, args: &Args) -> String {
        Mf2Texts::new(TEXTS, &self.locale)
            .say(&TextId::new(id), args)
            .text()
            .to_owned()
    }

    /// "3 hidden".
    pub fn hidden(&self, count: usize) -> String {
        self.say("board-hidden", &Args::new().count("count", count as u64))
    }

    /// "+ 5 more": the elements of a detail frame, or the facts of a card, beyond the limit.
    pub fn more(&self, count: usize) -> String {
        self.say("board-more", &Args::new().count("count", count as u64))
    }

    /// "Create 3 cards?".
    pub fn create_many(&self, count: usize) -> String {
        self.say(
            "board-create-many",
            &Args::new().count("count", count as u64),
        )
    }
}
