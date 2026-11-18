//! `drop-zone`: an area that takes dragged files and has a keyboard way to the same end.
//!
//! Pattern: Facade + Observer (`view.rs`: a clonable handle, `Live` draws it again) + Adapter. GNOME HIG: Drag and drop — every drag
//! needs an alternative that does not drag. The zone is a button (`role="button"`, a tab stop): Enter or Space calls the pick
//! handler, where the host opens its file chooser (`fs-components-filepicker`); the same handler is what a click does. While files
//! hover over it the zone says so (`over` state and words). Limit: Blinc's drop event carries no paths; `on_drop` only says that
//! files were dropped here, the host reads them from the window's drop list.

mod labels;
mod styles;
mod view;

pub use labels::{Labels, TEXTS};
pub use styles::STYLES;
pub use view::DropZone;
