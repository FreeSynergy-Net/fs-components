//! The pure state of a tooltip: hover with a delay, focus, Escape. Time is explicit, so a test needs no waiting.
//!
//! Pattern: State. No Blinc here.

/// The pause between the pointer coming to rest and the tooltip showing, in milliseconds, unless told otherwise.
pub const DEFAULT_DELAY_MS: u64 = 400;

/// What decides whether the bubble shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct State {
    pub delay_ms: u64,
    hover_since: Option<u64>,
    focus: bool,
    dismissed: bool,
    pinned: bool,
}

impl State {
    pub fn new(delay_ms: u64) -> Self {
        Self {
            delay_ms,
            hover_since: None,
            focus: false,
            dismissed: false,
            pinned: false,
        }
    }

    pub fn hover(&mut self, on: bool, now: u64) {
        if on {
            self.hover_since.get_or_insert(now);
        } else {
            self.hover_since = None;
            self.pinned = false; // the pointer left (touch: tapped elsewhere): the pressed bubble closes
        }
        self.settle();
    }

    pub fn focus(&mut self, on: bool) {
        self.focus = on;
        if !on {
            self.pinned = false; // blur closes a pressed bubble
        }
        self.settle();
    }

    /// A press on the trigger: the bubble stays until a second press, Escape, blur or the pointer leaving (touch has no hover).
    pub fn open(&mut self) {
        self.pinned = !self.pinned;
        self.dismissed = false;
    }

    /// Escape: closes the bubble until pointer and keyboard have both left.
    pub fn escape(&mut self) {
        self.dismissed = true;
        self.pinned = false;
    }

    /// Pointer and keyboard gone: the next visit may show it again.
    fn settle(&mut self) {
        if self.hover_since.is_none() && !self.focus {
            self.dismissed = false;
        }
    }

    /// A focus tooltip shows at once, a hover tooltip after the delay.
    pub fn visible(&self, now: u64) -> bool {
        !self.dismissed
            && (self.focus
                || self.pinned
                || self
                    .hover_since
                    .is_some_and(|t| now >= t.saturating_add(self.delay_ms)))
    }
}
