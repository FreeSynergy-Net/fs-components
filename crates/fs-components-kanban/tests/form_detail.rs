//! The frame at the side from fixed rows of the form, without a module: the path is the column name, the title the title, the
//! lines are the channels of the card; a channel nobody fills takes no place, an empty one is a line with an empty value ("set").
//!
//! Mutants: the path is the column key -> `the_path_is_the_column_name_and_the_title_the_title`; a channel without field and value
//! gets a line -> `a_channel_without_field_or_value_takes_no_place`; an empty channel of the module is dropped ->
//! `an_empty_channel_of_the_module_is_a_line_to_fill`; the adapter is not set on the view -> `a_click_on_a_card_opens_the_frame`;
//! a facts field of the same name is drawn twice -> `a_field_of_facts_wins_over_the_own_channel`.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

mod board_common;
mod common;

use std::path::PathBuf;
use std::sync::Arc;

use board_common::Rig;
use fs_components_core::form_read::{ChannelInfo, FieldInfo, FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView};
use fs_components_core::form_write::{FormWrite, Gesture, WriteArg, WriteDone, WriteRefused};
use fs_components_core::states::Load;
use fs_components_fields::Say;
use fs_components_kanban::{DetailContent, FormDetail, FormSource, View};
use fs_components_render::{Canvas, RenderError, draw_to_png};
use fs_components_styles::StyleId;

struct Host;

impl FormWrite for Host {
    fn can(&self, _: &str, _: Option<&str>) -> bool {
        true
    }
    fn may_fire(&self, gesture: Gesture<'_>, record: Option<&str>) -> Result<(), WriteRefused> {
        if self.can(common::channel_of(gesture), record) {
            Ok(())
        } else {
            Err(WriteRefused::new("locked"))
        }
    }

    fn fire(
        &self,
        gesture: Gesture<'_>,
        record: Option<&str>,
        args: &[WriteArg],
    ) -> Result<WriteDone, WriteRefused> {
        self.send(common::channel_of(gesture), record, args)
    }

    fn send(&self, _: &str, _: Option<&str>, _: &[WriteArg]) -> Result<WriteDone, WriteRefused> {
        Ok(WriteDone::default())
    }
}

fn channel(name: &str, fields: &[&str]) -> ChannelInfo {
    ChannelInfo {
        name: name.into(),
        fields: fields
            .iter()
            .map(|f| FieldInfo {
                name: (*f).into(),
                block: "field-text".into(),
            })
            .collect(),
        choices: Vec::new(),
    }
}

fn input() -> FormInput {
    let mut a = FormRow::of(
        "a",
        &[
            ("title", "Write the report"),
            ("column", "doing"),
            ("column-label", "Doing"),
            ("priority", "2"),
            ("assignee", "Ada"),
        ],
    );
    a.values
        .push(("facts".into(), vec!["2026-12-01".into(), "Ada".into()]));
    let b = FormRow::of("b", &[("title", "Plain"), ("column", "todo")]);
    FormInput {
        form: "cards-in-columns@1.4.0".into(),
        rows: vec![a, b],
        channels: vec![
            ChannelInfo {
                choices: vec![
                    ("todo".into(), "To do".into()),
                    ("doing".into(), "Doing".into()),
                ],
                ..channel("column", &[])
            },
            channel("facts", &["due", "owner"]),
            channel("description", &["notes"]),
            channel("assignee", &["owner"]),
        ],
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn detail(input: FormInput) -> FormDetail {
    let say: Arc<dyn Say> = Arc::new(|key: &str| format!("<{key}>"));
    FormDetail::new(
        Arc::new(FormSource::new(input, Arc::new(Host), say.clone())),
        say,
    )
}

fn lines(input: FormInput, card: &str) -> Vec<(String, String)> {
    detail(input)
        .frame(card)
        .expect("the card is there")
        .properties
}

#[test]
fn the_path_is_the_column_name_and_the_title_the_title() {
    let frame = detail(input()).frame("a").unwrap();
    assert_eq!(
        (frame.path.as_str(), frame.title.as_str()),
        ("Doing", "Write the report")
    );
    assert_eq!(
        detail(input()).frame("b").unwrap().path,
        "todo",
        "no label: the key"
    );
    assert!(
        detail(input()).frame("gone").is_none(),
        "a card that is gone has no frame"
    );
}

#[test]
fn the_card_shows_its_channels_with_the_names_of_the_host() {
    let got = lines(input(), "a");
    let pair = |n: &str, v: &str| (n.to_owned(), v.to_owned());
    // `owner` is also the assignee: it is drawn once, as the assignee line
    assert_eq!(
        got,
        vec![
            pair("<fact.due>", "2026-12-01"),
            pair("<channel.priority>", "2"),
            pair("<channel.assignee>", "Ada"),
            pair("<channel.description>", ""),
        ]
    );
}

#[test]
fn an_empty_channel_of_the_module_is_a_line_to_fill() {
    let got = lines(input(), "b");
    assert!(
        got.contains(&("<channel.description>".into(), String::new())),
        "{got:?}"
    );
    assert!(
        got.contains(&("<fact.due>".into(), String::new())),
        "{got:?}"
    );
}

#[test]
fn a_channel_without_field_or_value_takes_no_place() {
    let got = lines(input(), "b");
    let names: Vec<&str> = got.iter().map(|(n, _)| n.as_str()).collect();
    assert!(
        !names.contains(&"channel.priority") && !names.contains(&"channel.state"),
        "{names:?}"
    );
}

#[test]
fn a_field_of_facts_wins_over_the_own_channel() {
    let mut i = input();
    i.channels[1] = channel("facts", &["due", "priority"]);
    i.rows[0] = with_facts(&["x", "9"]);
    let got = lines(i, "a");
    assert_eq!(
        got.iter().filter(|(n, _)| n.contains("priority")).count(),
        1,
        "{got:?}"
    );
}

#[test]
fn several_values_of_one_field_do_not_get_a_wrong_name() {
    let mut i = input();
    i.rows[0] = with_facts(&["x", "y", "z"]);
    let got = lines(i, "a");
    assert_eq!(
        got.iter().filter(|(n, _)| n == "<channel.facts>").count(),
        3,
        "{got:?}"
    );
}

fn mounted() -> (Rig, Arc<dyn fs_components_core::form_view::Mounted>) {
    let ctx = DrawCtx {
        place: fs_components_core::form_view::PlaceInfo::default(),
        look: (StyleId::Admin, false),
        say: Arc::new(str::to_owned),
        locale: fs_components_words::Locale::english(),
        write: Arc::new(Host),
        reread: Arc::new(input),
    };
    let view: Arc<dyn fs_components_core::form_view::Mounted> = Arc::from(View.mount(input(), ctx));
    (Rig::mounted(view.clone(), 1100.0, 640.0), view)
}

#[test]
fn a_click_on_a_card_opens_the_frame() {
    let (mut rig, view) = mounted();
    let _ = &view;
    assert!(!rig.has("fs-board-detail"), "closed at first");
    rig.click("fs-board-card-a");
    assert!(rig.has("fs-board-detail"), "the frame stands at the side");
}

#[test]
fn the_frame_is_drawn_to_a_picture() {
    let (mut rig, view) = mounted();
    rig.click("fs-board-card-a");
    let dir = std::env::var_os("FS_COMP_SHOTS").map_or_else(
        || std::env::temp_dir().join(format!("fs-detail-{}", std::process::id())),
        PathBuf::from,
    );
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("board-detail-form-light.png");
    let canvas = Canvas::new(1100, 640, fs_components_kanban::styles());
    match draw_to_png(&canvas, move |w, h| view.build(w, h), || {}, &path) {
        Err(RenderError::NoGraphics(why)) => eprintln!("skipped: {why}"),
        other => {
            other.unwrap();
            let picture = image::open(&path).unwrap().to_rgba8();
            let colours: std::collections::HashSet<_> = picture.pixels().map(|p| p.0).collect();
            assert!(colours.len() > 20, "something was drawn");
            if std::env::var_os("FS_COMP_SHOTS").is_none() {
                std::fs::remove_file(&path).unwrap();
            }
        }
    }
}

/// A row whose channel `facts` carries several values (one per field of the module).
fn with_facts(values: &[&str]) -> FormRow {
    let mut row = FormRow::of(
        "a",
        &[("title", "t"), ("column", "todo"), ("priority", "9")],
    );
    row.values.push((
        "facts".into(),
        values.iter().map(|v| (*v).to_owned()).collect(),
    ));
    row
}

#[test]
fn a_name_the_host_has_no_words_for_is_the_bare_name() {
    let say: Arc<dyn Say> = Arc::new(str::to_owned);
    let source = FormSource::new(input(), Arc::new(Host), say.clone());
    let got = FormDetail::new(Arc::new(source), say).frame("a").unwrap();
    let names: Vec<&str> = got.properties.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["due", "priority", "assignee", "description"]);
}
