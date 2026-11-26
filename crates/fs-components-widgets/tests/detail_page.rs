//! `detail-page`: the page of one record read from `single-record` (values by path, several = several cards under ONE label).
//!
//! Tests: pure reading tests over `FormInput` and the sheet; the element is looked at by its ids (`<name>-field-<section>-<field>[-<n>]`).
//!
//! Mutants (each must turn a test red):
//! - the last value of a repeated field wins ("never keep only one") -> `two_emails_are_two_values_of_one_field`
//! - a value is taken by position, a missing field shifts the rest   -> `a_field_without_a_value_shifts_nothing`
//! - an empty value is drawn as a card, or a further value adds none -> `empty_values_are_not_drawn_and_a_further_value_is_a_card`
//! - an older Minor (1.0.0) is refused                               -> `a_record_of_the_old_minor_is_read_the_same_way`

mod common;

use blinc_layout::prelude::RenderTree;
use fs_components_core::form_read::{ChannelInfo, FieldInfo, FormInput, FormRow};
use fs_components_core::states::Load;
use fs_components_widgets::detail_page::form_view::{FIELDS, sheet};
use fs_components_widgets::detail_page::{Field, FieldKind, Section, Sheet, page};

fn field(name: &str, block: &str) -> FieldInfo {
    FieldInfo {
        name: name.to_owned(),
        block: block.to_owned(),
        label: String::new(),
    }
}

fn input(form: &str, values: &[(&str, &str)]) -> FormInput {
    let map = vec![(
        FIELDS.to_owned(),
        ["organizations", "emails", "phones"]
            .map(str::to_owned)
            .to_vec(),
    )];
    let row = FormRow::from_values(
        "amara",
        Vec::new(),
        &map,
        values
            .iter()
            .map(|(p, t)| (*p, *t))
            .chain([("label", "Amara Okafor")].map(|(p, t)| (p, t))),
    )
    .with_values("label", &[("name", "Amara Okafor")]);
    FormInput {
        form: form.to_owned(),
        rows: vec![row],
        channels: vec![ChannelInfo {
            name: FIELDS.to_owned(),
            fields: vec![
                field("organizations", "field-text"),
                field("emails", "field-email"),
                field("phones", "field-phone"),
            ],
            choices: Vec::new(),
        }],
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn read(input: &FormInput) -> Sheet {
    sheet(input, &input.rows[0], &|k: &str| k.to_owned())
}

fn fields(s: &Sheet) -> &[Field] {
    &s.sections[0].fields
}

#[test]
fn two_emails_are_two_values_of_one_field() {
    let i = input(
        "single-record@1.2.0",
        &[
            ("emails", "amara@example.org"),
            ("emails", "a.okafor@work.example"),
        ],
    );
    let s = read(&i);
    assert_eq!(s.title, "Amara Okafor");
    let emails = fields(&s).iter().find(|f| f.label == "emails").unwrap();
    assert_eq!(
        emails.values,
        ["amara@example.org", "a.okafor@work.example"]
    );
    assert_eq!(
        emails.kind,
        FieldKind::Mail,
        "the kind comes from the block of the field"
    );
}

#[test]
fn a_field_without_a_value_shifts_nothing() {
    // `organizations` has no value: the e-mail must not slide into its place, the phone stays a phone
    let i = input(
        "single-record@1.2.0",
        &[
            ("emails", "amara@example.org"),
            ("phones", "+49 30 1234567"),
        ],
    );
    let s = read(&i);
    let labels: Vec<&str> = fields(&s).iter().map(|f| f.label.as_str()).collect();
    assert_eq!(labels, ["emails", "phones"], "no field `organizations`");
    assert_eq!(fields(&s)[1].values, ["+49 30 1234567"]);
    assert_eq!(fields(&s)[1].kind, FieldKind::Phone);
}

#[test]
fn a_record_of_the_old_minor_is_read_the_same_way() {
    // 1.0.0 has one value per field; a tolerant reader takes what comes, also if it were repeated
    let one = read(&input(
        "single-record@1.0.0",
        &[("emails", "amara@example.org")],
    ));
    assert_eq!(fields(&one)[0].values, ["amara@example.org"]);
    let many = read(&input(
        "single-record@1.0.0",
        &[("emails", "a@x.org"), ("emails", "b@x.org")],
    ));
    assert_eq!(fields(&many)[0].values.len(), 2, "never keep only one");
}

#[test]
fn a_record_without_fields_is_a_page_with_a_name() {
    let s = read(&input("single-record@1.2.0", &[]));
    assert_eq!(s.title, "Amara Okafor");
    assert_eq!(fields(&s), []);
}

#[test]
fn empty_values_are_not_drawn_and_a_further_value_is_a_card() {
    let _g = common::serial();
    let s = Sheet::new(
        "Amara Okafor",
        vec![Section::new(
            "",
            vec![
                Field::many(
                    "E-mail",
                    vec!["a@x.org".into(), "b@x.org".into(), String::new()],
                )
                .kind(FieldKind::Mail),
                Field::new("Phone", "  "),
            ],
        )],
    );
    // an empty value and a field of only empty values are not drawn: ids -0-0 and -0-0-1 exist, -0-0-2 and the phone do not
    let mut tree = RenderTree::from_element(&page(&s, "p", "", &|_, _| Vec::new()));
    tree.compute_layout(400.0, 600.0);
    let has = |id: &str| tree.element_registry().get(id).is_some();
    assert!(has("p-field-0-0") && has("p-field-0-0-1"));
    assert!(!has("p-field-0-0-2") && !has("p-field-0-1"));
}
