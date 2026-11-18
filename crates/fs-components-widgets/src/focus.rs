//! Giving the keyboard to an element: a request to the host.
//!
//! Pattern: Strategy — a widget cannot move the keyboard itself (Blinc keeps focus in the window's router, which no widget sees),
//! so it names the element that should have it and the host does it. `alertdialog` asks for its first stop, `popover` for its
//! trigger when it closes (WAI-ARIA APG, dialog: focus returns to the invoking element).

use std::sync::Arc;

/// What the host runs to give the keyboard to the element with this id.
pub type FocusTo = Arc<dyn Fn(&str) + Send + Sync>;
