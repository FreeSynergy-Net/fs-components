//! Where the messages of one language come from.
//!
//! Pattern: Strategy — `Messages` is one language's store of messages; a message is looked up by id and filled by the caller.
//! `Packed` reads a catalog the packer wrote (the runtime way, no `compile`); `Compiled` compiles `.mf2` sources on first use
//! (the old way, feature `compile`, until wave C moves its callers).

use mf2_catalog::MsgId;
use mf2_catalog::{Catalog, Manifest};

/// What fills a message: the catalog, the id of the message and the names of its variables (in the order the formatter wants its values).
pub(crate) type Fill<'a> = dyn FnMut(&Catalog, MsgId, &[String]) -> Option<String> + 'a;

/// A language's messages. `with` calls `fill` with the catalog, the id of the message and the names of its variables (in the
/// order the formatter wants its values); `None` when the language has no such message.
pub(crate) trait Messages: Send + Sync {
    fn with(&self, id: &str, fill: &mut Fill<'_>) -> Option<String>;
}

/// A catalog of the packer with its manifest.
pub(crate) struct Packed {
    pub(crate) catalog: Catalog,
    pub(crate) manifest: std::sync::Arc<Manifest>,
}

impl Messages for Packed {
    fn with(&self, id: &str, fill: &mut Fill<'_>) -> Option<String> {
        let at = self
            .manifest
            .ids
            .binary_search_by(|x| x.as_bytes().cmp(id.as_bytes()))
            .ok()?;
        let msg = self.manifest.msg_id(id)?;
        fill(&self.catalog, msg, self.manifest.slots.get(at)?)
    }
}

#[cfg(feature = "compile")]
pub(crate) use compiled::Compiled;

#[cfg(feature = "compile")]
mod compiled {
    use std::collections::HashMap;
    use std::sync::{Mutex, PoisonError};

    use super::{Fill, Messages};

    /// `.mf2` messages compiled with `mf2::compile_str` the first time each is asked for, then kept.
    pub(crate) struct Compiled {
        pub(crate) tag: String,
        pub(crate) messages: HashMap<String, String>,
        pub(crate) cache: Mutex<HashMap<String, Option<mf2::Compiled>>>,
    }

    impl Messages for Compiled {
        fn with(&self, id: &str, fill: &mut Fill<'_>) -> Option<String> {
            let source = self.messages.get(id)?;
            let mut cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
            let compiled = cache
                .entry(id.to_owned())
                .or_insert_with(|| mf2::compile_str(source, &self.tag).ok())
                .as_ref()?;
            fill(
                &compiled.catalog,
                mf2::Compiled::ID,
                compiled.manifest.slots.first()?,
            )
        }
    }
}
