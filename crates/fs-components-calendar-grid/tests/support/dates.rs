//! Test support: the context of a host that has the date package of one language.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic, dead_code)]

use std::sync::Arc;

use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, PlaceInfo};
use fs_components_core::form_write::NoWrite;
use fs_components_core::states::Load;
use fs_components_mf2::{DateDir, date_names};
use fs_components_styles::StyleId;
use fs_components_words::{Locale, with_week_start};

/// A form of `rows`.
pub fn input(rows: Vec<FormRow>) -> FormInput {
    FormInput {
        form: "bars-in-time@1.0.0".to_owned(),
        rows,
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

/// What a host does once per language: the package of the mf2 test data, the setting of the person on the locale. The host has
/// no sentences of its own (it answers with the key), so the words come from the catalog of the calendar components.
pub fn host_ctx(
    tag: &str,
    week: &str,
    place: PlaceInfo,
    dark: bool,
    rows: Vec<FormRow>,
) -> DrawCtx {
    let dir = DateDir::new(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../fs-components-mf2/tests/data/packed"),
    );
    let locale = with_week_start(&Locale::parse(tag).unwrap(), week);
    let dates = date_names(&dir, &locale);
    DrawCtx::new(
        place,
        (StyleId::Admin, dark),
        Arc::new(|k: &str| k.to_owned()),
        locale,
        Arc::new(NoWrite),
        Arc::new(move || input(rows.clone())),
    )
    .with_dates(dates)
}
