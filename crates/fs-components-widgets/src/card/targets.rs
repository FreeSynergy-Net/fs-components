//! What the container answers and what the card tells its owner.
//!
//! Pattern: Strategy ([`DropTargets`]: the container says which place lies under the pointer and whether it takes the card; the card
//! only asks) + Observer ([`CardEvents`]: what the card tells its owner, every method has a default). The names of the three
//! gestures are the slots of the block `card` ([`ACTIVATE`], [`MENU`], [`DROP`]): the host binds an `[[action]]` by that trigger, the
//! component knows no channel and no command.

use super::gesture::Over;
use super::refs::{CardRef, Carry, Target};
use crate::answer::{Answer, Reason};

/// The trigger of a click without a carry and of Enter (slot `activate` of the block `card`).
pub const ACTIVATE: &str = "activate";
/// The trigger of the context menu (slot `menu`).
pub const MENU: &str = "menu";
/// The trigger of a card put down at a place (slot `drop`, arguments [`Target::ARGS`]).
pub const DROP: &str = "drop";

/// Pattern: Strategy - the container answers for its places. The methods run while the card holds its own state: they must not call
/// back into the card.
pub trait DropTargets: Send + Sync {
    /// The place under `window` (window coordinates) when `carried` is carried; `None` = over no place.
    fn target_at(&self, window: (f32, f32), carried: &CardRef) -> Option<Target>;

    /// Whether `carried` may land at `to`; the reason is shown at once, while the card is still over the place.
    ///
    /// # Errors
    /// The place does not take the card: the reason in the reader's words.
    fn can_drop(&self, _carried: &CardRef, _to: &Target) -> Result<(), Reason> {
        Ok(())
    }
}

/// Pattern: Observer - what the card tells its owner. Every method has a default (nothing happens, a drop is refused).
pub trait CardEvents: Send + Sync {
    /// A click without a carry, or Enter.
    fn activate(&self, _card: &CardRef) {}

    /// The context menu was asked for (right click, Shift+F10, the Menu key) at `at` (window coordinates).
    fn menu(&self, _card: &CardRef, _at: (f32, f32)) {}

    /// The card was put down at `to`. The program says yes, no with a reason, or asks first (`Answer::Ask`, the same request comes
    /// again once the reader said yes).
    fn dropped(&self, _carry: &Carry, _to: &Target) -> Answer {
        Answer::Refused(Reason::new(
            "card-drop-unsupported",
            "This place does not take cards.",
        ))
    }

    /// The look of the card changed (carried, held, locked): the owner draws it again.
    fn changed(&self, _card: &CardRef) {}
}

/// The events of a card whose owner wants none (an installer tile that is only clicked can still name `activate`).
#[derive(Clone, Copy, Debug, Default)]
pub struct NoEvents;

impl CardEvents for NoEvents {}

/// A [`DropTargets`] seen as the [`Over`] a gesture asks. The gesture calls with window coordinates for the card.
pub(super) struct Via<'a>(pub(super) &'a dyn DropTargets);

impl Over<CardRef, Target> for Via<'_> {
    fn place(&self, x: f32, y: f32, skip: Option<&CardRef>) -> Option<Target> {
        skip.and_then(|card| self.0.target_at((x, y), card))
    }
}
