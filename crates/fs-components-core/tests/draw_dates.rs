//! Pattern: none (test). `DrawCtx` carries the date names: neutral without a package, the language of the package when the
//! host injects it, another language gives other names, `-u-fw` moves the week start.

use std::sync::Arc;

use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, PlaceInfo};
use fs_components_core::form_write::NoWrite;
use fs_components_core::states::Load;
use fs_components_mf2::{DateDir, date_names};
use fs_components_styles::StyleId;
use fs_components_words::{Locale, Weekday, Width, with_week_start};

fn ctx() -> DrawCtx {
    DrawCtx::new(
        PlaceInfo::default(),
        (StyleId::Admin, false),
        Arc::new(|k: &str| k.to_owned()),
        Locale::english(),
        Arc::new(NoWrite),
        Arc::new(input),
    )
}

/// What the host does once per language: the package of the mf2 test data, the person's week start on the locale.
fn host_ctx(tag: &str, week_start: &str) -> DrawCtx {
    let dir = DateDir::new(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../fs-components-mf2/tests/data/packed"),
    );
    let locale = with_week_start(&Locale::parse(tag).unwrap(), week_start);
    let dates = date_names(&dir, &locale);
    ctx().with_dates(dates)
}

#[test]
fn without_a_package_the_names_are_neutral() {
    let c = ctx();
    assert_eq!(c.dates.weekday(Weekday::Monday, Width::Short), "Mon");
    assert_eq!(c.dates.month_title(2026, 11, Width::Long), "2026-11");
    assert_eq!(c.dates.first_weekday(), Weekday::Monday);
}

#[test]
fn the_language_of_the_package_gives_its_names_and_another_language_other_ones() {
    let de = host_ctx("de", "auto");
    let en = host_ctx("en", "auto");
    assert_eq!(de.dates.month_title(2026, 11, Width::Long), "November 2026");
    assert_eq!(de.dates.weekday(Weekday::Monday, Width::Long), "Montag");
    assert_eq!(en.dates.weekday(Weekday::Monday, Width::Long), "Monday");
    assert_ne!(
        de.dates.day_title(2026, 11, 26, Width::Long),
        en.dates.day_title(2026, 11, 26, Width::Long)
    );
}

#[test]
fn the_week_start_setting_reaches_the_names_and_a_clone_shares_them() {
    assert_eq!(
        host_ctx("de", "auto").dates.first_weekday(),
        Weekday::Monday
    );
    assert_eq!(host_ctx("de", "sun").dates.first_weekday(), Weekday::Sunday);
    assert_eq!(
        host_ctx("en-US", "auto").dates.first_weekday(),
        Weekday::Sunday
    );
    assert_eq!(
        host_ctx("en-US", "mon").dates.first_weekday(),
        Weekday::Monday
    );
    let c = host_ctx("de", "auto");
    assert!(
        Arc::ptr_eq(&c.dates, &c.clone().dates),
        "one set of names per language"
    );
}

fn input() -> FormInput {
    FormInput {
        form: "bars-in-time@1.1.0".to_owned(),
        rows: Vec::new(),
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}
