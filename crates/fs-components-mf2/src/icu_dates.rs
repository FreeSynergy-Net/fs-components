//! Attaching the date table of a language to its catalog.
//!
//! Pattern: Strategy (the table is the one a `DateData` hands out); this file only checks and attaches it.
//!
//! `Catalog::with_server_data` of `mf2-catalog` takes `&'static [u8]` and it consumes the catalog (an error loses it).
//! Therefore: (1) the table is leaked ONCE per distinct table, never per call (`LEAKED` hands the same slice again), (2) a
//! table over `LIMIT` bytes is refused before it is leaked (a real one is 1 to 4 KB; about 1.4 KB for all Gregorian forms),
//! (3) the catalog is built from its bytes for each try, so a broken table leaves the plain catalog. The leak is bounded by
//! the number of distinct tables a program loads (one per language, ~1.4 KB each) and is open on purpose: see
//! `.scratch/woche-11-23/form/anfrage-mf2-owned-data.md` (a request to the `mf2` maintainers for owning data).

use std::sync::{Mutex, PoisonError};

use mf2_catalog::Catalog;

/// The largest table taken (bytes). The packer writes 1 to 4 KB; anything bigger is no date table of ours.
const LIMIT: usize = 64 * 1024;

/// The tables leaked so far (each distinct table once).
static LEAKED: Mutex<Vec<&'static [u8]>> = Mutex::new(Vec::new());

/// `table` as `'static` data: the one leaked copy of these bytes (made on the first call for them).
// BOX-LEAK (open, bounded): once per distinct date table; `with_server_data` wants `&'static [u8]`.
fn leaked(table: &[u8]) -> &'static [u8] {
    let mut all = LEAKED.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(known) = all.iter().find(|k| **k == table) {
        return known;
    }
    let slice: &'static [u8] = Box::leak(table.to_vec().into_boxed_slice());
    all.push(slice);
    slice
}

/// The catalog from `bytes` with `table` as its date data. `None` when there is no table, it is too big, it is no valid
/// table, or the catalog itself does not load; the caller then keeps the plain catalog and the neutral functions.
pub(crate) fn with_table(
    bytes: &[u8],
    manifest_hash: u64,
    table: Option<&[u8]>,
) -> Option<Catalog> {
    let table = table.filter(|t| !t.is_empty() && t.len() <= LIMIT)?;
    let catalog = Catalog::new(bytes.to_vec(), manifest_hash).ok()?;
    catalog.with_server_data(leaked(table)).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_table_is_leaked_once() {
        let a = leaked(b"table one");
        let b = leaked(b"table one");
        assert!(std::ptr::eq(a, b), "the same slice again, no second leak");
        let c = leaked(b"table two");
        assert!(!std::ptr::eq(a, c));
    }

    #[test]
    fn a_table_that_is_too_big_or_empty_is_not_taken() {
        // Own test, own table: the counter is global, so only the refusal is checked, not the count.
        // A real catalog, so that only the size decides.
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/packed");
        let manifest =
            mf2_catalog::Manifest::read(&std::fs::read(dir.join("manifest.mf2m")).unwrap())
                .unwrap();
        let file = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .find(|e| e.file_name().to_string_lossy().starts_with("de."))
            .unwrap();
        let bytes = std::fs::read(file.path()).unwrap();
        let big = vec![7_u8; LIMIT + 1];
        assert!(with_table(&bytes, manifest.hash(), Some(&big)).is_none());
        assert!(with_table(&bytes, manifest.hash(), Some(&[])).is_none());
        let all = LEAKED.lock().unwrap();
        assert!(
            !all.iter().any(|k| k.len() > LIMIT),
            "refused before the leak"
        );
    }
}
