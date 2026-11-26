//! The form `rows` of a person as a picture (table and grid), light and dark: two e-mails of Amara stand as two lines in ONE cell
//! under their head, the empty `organizations` cell stays empty and nothing slides into it. Pictures are kept when `FS_COMP_SHOTS`
//! names a directory.
//!
//! Pattern: Test double (a `NoWrite` host and a `say` with the words of the fields) over rows built by `FormRow::from_values`.
//! Pixel tests; they need a graphics adapter.

#![allow(clippy::pedantic)]

mod common;

use std::sync::Arc;

use common::{background, draw, ink, serial, set_dark};
use fs_components_core::form_read::{ChannelInfo, FieldInfo, FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView};
use fs_components_core::form_write::NoWrite;
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_words::Locale;

const FIELDS: [(&str, &str); 3] = [
    ("organizations", "Organization"),
    ("emails", "E-mail"),
    ("phones", "Phone"),
];

fn input() -> FormInput {
    let map = vec![(
        "columns".to_owned(),
        FIELDS
            .iter()
            .map(|(n, _)| (*n).to_owned())
            .collect::<Vec<_>>(),
    )];
    // Amara: two e-mails, no organization (the module leaves it out), one phone; Tariq: everything once
    let amara = FormRow::from_values(
        "amara",
        Vec::new(),
        &map,
        [
            ("emails", "amara@zolenko.example"),
            ("emails", "a.zolenko@work.example"),
            ("phones", "+49 30 1234"),
        ],
    )
    .with_values("label", &[("label", "Amara Zolenko")]);
    let tariq = FormRow::from_values(
        "tariq",
        Vec::new(),
        &map,
        [
            ("organizations", "Helfa"),
            ("emails", "tariq@lindqvist.example"),
            ("phones", "+46 8 555"),
        ],
    )
    .with_values("label", &[("label", "Tariq Lindqvist")]);
    FormInput {
        form: "rows@1.0.0".to_owned(),
        rows: vec![amara, tariq],
        channels: vec![ChannelInfo {
            name: "columns".to_owned(),
            fields: FIELDS
                .iter()
                .map(|(n, _)| FieldInfo {
                    name: (*n).to_owned(),
                    block: "field-text".to_owned(),
                    label: format!("person.{n}"),
                })
                .collect(),
            choices: Vec::new(),
        }],
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn ctx(dark: bool) -> DrawCtx {
    DrawCtx::new(
        fs_components_core::form_view::PlaceInfo::default(),
        (StyleId::Admin, dark),
        Arc::new(|k| {
            FIELDS
                .iter()
                .find(|(n, _)| k == format!("person.{n}"))
                .map_or_else(|| k.to_owned(), |(_, w)| (*w).to_owned())
        }),
        Locale::english(),
        Arc::new(NoWrite),
        Arc::new(input),
    )
}

#[test]
fn the_table_of_people_has_the_values_under_their_heads_in_both_schemes() {
    let _g = serial();
    let mut shots = Vec::new();
    for dark in [false, true] {
        set_dark(dark);
        let mounted = fs_components_widgets::table::form_view::View.mount(input(), ctx(dark));
        let name = if dark {
            "people_rows_dark.png"
        } else {
            "people_rows_light.png"
        };
        let img = draw(mounted.build(900.0, 200.0), 900, 200, name);
        let bg = background(&img);
        assert!(ink(&img, (0, 0, 900, 200), bg) > 1500, "dark={dark}");
        // Amara has no organization: its cell is empty and her e-mails did not slide into it
        assert_eq!(
            ink(&img, (243, 34, 217, 32), bg),
            ink(&img, (243, 120, 217, 32), bg),
            "the empty cell looks like the empty ground below the rows, dark={dark}"
        );
        // her two e-mails stand in the cell under their head, Tariq's organization under its own
        assert!(
            ink(&img, (468, 34, 222, 32), bg) > 150,
            "two e-mails, dark={dark}"
        );
        assert!(
            ink(&img, (243, 76, 217, 18), bg) > 20,
            "organization, dark={dark}"
        );
        shots.push(img);
    }
    set_dark(false);
    assert_ne!(shots[0], shots[1], "light and dark differ");
}
