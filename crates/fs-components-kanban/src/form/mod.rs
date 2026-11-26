//! The forms adapter of the board (feature `form`): `FormInput`/`FormWrite` of the host on the ports of the core
//! ([`crate::BoardSource`], [`crate::BoardEvents`]), and the [`View`] the host mounts. The channel names (`move`, `create`,
//! `column.*`) live only in this directory; the core knows none of them.
//!
//! Pattern: Adapter (Ports and Adapters) – the core is the hexagon, this directory is its one adapter to `fs-components-core`.

mod ask;
mod carried;
mod detail;
mod dialogs;
mod events;
mod read;
mod source;
mod view;
mod words;

pub use carried::carried_channels;
pub use detail::FormDetail;
pub use events::{COLUMN_CREATE, COLUMN_LIMIT, COLUMN_RENAME, FormEvents, LANE, LIMIT, NAME};
pub use read::columns_of;
pub use source::FormSource;
pub use view::View;
pub use words::{KEYS as WORD_KEYS, REFUSED_CODES, TEXTS as WORD_TEXTS, id_of as word_id};
