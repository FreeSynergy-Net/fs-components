//! Shared parts of the colour pictures: rows, the context, the drawing of one grid.
#![allow(clippy::pedantic, dead_code)]

use crate::ui;

use std::sync::Arc;

use fs_components_calendar_grid::Day;
use fs_components_calendar_grid::form_view::View;
use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, PlaceInfo};
use fs_components_core::form_write::NoWrite;
use fs_components_core::states::Load;
use fs_components_fields::Look;
use fs_components_styles::{Rgba, StyleId};
use fs_components_words::Locale;
use ui::{draw_tree, layout};

pub const W: f32 = 900.0;
pub const H: f32 = 640.0;

/// Six all-day rows of two calendars (`home` on `a`, `c`, `e`; `work` on `b`, `d`, `f`), their colours as the module writes them.
pub fn rows(home: Option<&str>, work: Option<&str>) -> Vec<FormRow> {
    let make = |key: &str, label: &str, day: &str, lane: &str, color: Option<&str>| {
        let mut pairs = vec![
            ("label", label),
            ("start", day),
            ("end", day),
            ("lane", lane),
            ("all-day", "true"),
        ];
        if let Some(c) = color {
            pairs.push(("color", c));
        }
        FormRow::of(key, &pairs)
    };
    vec![
        make("a", "Dentist", "2026-11-09", "home", home),
        make("b", "Planning", "2026-11-10", "work", work),
        make("c", "Birthday", "2026-11-12", "home", home),
        make("d", "Release", "2026-11-12", "work", work),
        make("e", "Market", "2026-11-17", "home", home),
        make("f", "Review", "2026-11-18", "work", work),
    ]
}

pub fn input(rows: Vec<FormRow>) -> FormInput {
    FormInput {
        form: "bars-in-time@1.1.0".to_owned(),
        rows,
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

pub fn ctx(style: StyleId, dark: bool, place: PlaceInfo) -> DrawCtx {
    DrawCtx::new(
        place,
        (style, dark),
        Arc::new(|k| k.to_owned()),
        Locale::english(),
        Arc::new(NoWrite),
        Arc::new(|| input(rows(None, None))),
    )
}

/// What one picture shows: the style, the appearance and the view.
#[derive(Clone, Copy)]
pub struct Shot {
    pub style: StyleId,
    pub dark: bool,
    pub agenda: bool,
}

pub struct Drawn {
    pub img: image::RgbaImage,
    pub tree: blinc_layout::prelude::RenderTree,
}

/// The grid over `rows`, laid out and (with fonts) drawn as `name`.
pub fn draw(rows: Vec<FormRow>, shot: Shot, name: &str) -> Option<Drawn> {
    ui::set_dark(shot.dark);
    let place = if shot.agenda {
        PlaceInfo::default().with_view("agenda")
    } else {
        PlaceInfo::default()
    };
    let mounted = View.mount_on(
        &input(rows),
        ctx(shot.style, shot.dark, place),
        Day::new(2026, 11, 10).unwrap(),
    );
    let tree = layout(mounted.build(W, H), W, H);
    ui::has_fonts().then(|| {
        let img = draw_tree(&tree, W as u32, H as u32, name);
        Drawn { img, tree }
    })
}

/// The pixel of the stripe of the item `id` (3 px in, half way down).
pub fn stripe(d: &Drawn, id: &str) -> [u8; 4] {
    let node = d.tree.query_by_id(id).unwrap_or_else(|| panic!("{id}"));
    let b = d.tree.get_absolute_bounds(node).expect("bounds");
    d.img
        .get_pixel((b.x + 3.0) as u32, (b.y + b.height / 2.0) as u32)
        .0
}

/// The pixel in the middle of the element `id` (the stripe of an agenda row).
pub fn centre(d: &Drawn, id: &str) -> [u8; 4] {
    let node = d.tree.query_by_id(id).unwrap_or_else(|| panic!("{id}"));
    let b = d.tree.get_absolute_bounds(node).expect("bounds");
    d.img
        .get_pixel((b.x + b.width / 2.0) as u32, (b.y + b.height / 2.0) as u32)
        .0
}

/// The pixel of the empty right part of the item `id` (its ground).
pub fn ground(d: &Drawn, id: &str) -> [u8; 4] {
    let node = d.tree.query_by_id(id).unwrap_or_else(|| panic!("{id}"));
    let b = d.tree.get_absolute_bounds(node).expect("bounds");
    d.img
        .get_pixel((b.x + b.width - 5.0) as u32, (b.y + b.height / 2.0) as u32)
        .0
}

pub fn rgb(c: Rgba) -> [u8; 4] {
    [c.r, c.g, c.b, 255]
}

pub fn look(shot: Shot) -> Look {
    Look::new(shot.style, shot.dark)
}
