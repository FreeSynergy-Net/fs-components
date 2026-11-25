//! `PlaceInfo` and its field in `DrawCtx`: default is "nothing asked", a clone keeps the values.

use std::sync::Arc;

use fs_components_core::form_read::FormInput;
use fs_components_core::form_view::{DrawCtx, PlaceInfo};
use fs_components_core::form_write::NoWrite;
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_words::Locale;

fn ctx(place: PlaceInfo) -> DrawCtx {
    DrawCtx {
        place,
        look: (StyleId::Admin, false),
        say: Arc::new(|k: &str| k.to_string()),
        locale: Locale::english(),
        write: Arc::new(NoWrite),
        reread: Arc::new(input),
    }
}

#[test]
fn default_asks_for_nothing() {
    let p = PlaceInfo::default();
    assert_eq!(p.view, None);
    assert_eq!(p.variant, None);
    assert_eq!(ctx(PlaceInfo::default()).place, PlaceInfo::default());
}

#[test]
fn builders_set_one_side_only() {
    let v = PlaceInfo::default().with_view("agenda");
    assert_eq!(
        (v.view.as_deref(), v.variant.as_deref()),
        (Some("agenda"), None)
    );
    let w = PlaceInfo::default().with_variant("compact");
    assert_eq!(
        (w.view.as_deref(), w.variant.as_deref()),
        (None, Some("compact"))
    );
}

#[test]
fn clone_of_the_context_keeps_the_place() {
    let c = ctx(PlaceInfo::default()
        .with_view("month")
        .with_variant("color-3"));
    let d = c.clone();
    assert_eq!(d.place.view.as_deref(), Some("month"));
    assert_eq!(d.place.variant.as_deref(), Some("color-3"));
    assert_eq!(c.place, d.place);
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
