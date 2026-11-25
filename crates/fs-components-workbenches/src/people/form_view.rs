//! The people workbench as a [`FormView`] of the form `rows`: the channel `label` names the person, the values of `columns` are the
//! fields of the sheet (named by the fields of the module that fill the channel). Read only: the source can do nothing else (no edit,
//! no create), so the workbench shows those controls locked, never working.
//!
//! The widget holds the state (selection, search, "more"), so the handle keeps it over a rebuild of the page and makes the widget
//! anew only when the rows change.
//!
//! Pattern: Abstract Factory (`View`) + Adapter (`PeopleSource` over the rows of a form).

use std::sync::{Arc, Mutex, PoisonError};

use crate::SourceError;
use blinc_layout::div::{Div, div};
use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView, Mounted};

use super::{
    PeopleCapabilities, PeopleLabels, PeopleSource, PeopleWidget, PersonField, PersonRow,
    PersonSection, PersonSheet,
};

/// The widget this view is (`widget` of the block `contact-row`).
pub const WIDGET: &str = "PeopleWidget";

/// The least height of the workbench, in pixels.
const MIN_H: f32 = 380.0;

/// The view of the block `contact-row`.
#[derive(Clone, Copy, Debug, Default)]
pub struct View;

impl FormView for View {
    fn widget(&self) -> &'static str {
        WIDGET
    }

    fn mount(&self, input: FormInput, _ctx: DrawCtx) -> Box<dyn Mounted> {
        let widget = widget_of(&input);
        Box::new(MountedPeople(Mutex::new((input, widget))))
    }
}

fn widget_of(input: &FormInput) -> PeopleWidget {
    PeopleWidget::compact(Arc::new(RowPeople::of(input)), PeopleLabels::default())
}

struct MountedPeople(Mutex<(FormInput, PeopleWidget)>);

impl Mounted for MountedPeople {
    fn build(&self, width: f32, height: f32) -> Div {
        let widget = self
            .0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .1
            .clone();
        let height = height.max(MIN_H);
        div().w(width).h(height).child(widget.build(width, height))
    }

    fn refresh(&self, input: FormInput) {
        let mut held = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if held.0.rows != input.rows || held.0.channels != input.channels {
            let widget = widget_of(&input);
            *held = (input, widget);
        }
    }
}

/// The people the rows of a form give, for the contact list.
struct RowPeople {
    rows: Vec<FormRow>,
    /// The names of the fields behind the channel `columns`, in order: the labels of the sheet.
    fields: Vec<String>,
    /// The name of the field behind the channel `state`: the label of the last field of the sheet.
    state: Option<String>,
}

impl RowPeople {
    fn of(input: &FormInput) -> Self {
        Self {
            rows: input.rows.clone(),
            fields: input.channel("columns").map_or_else(Vec::new, |c| {
                c.fields.iter().map(|f| f.name.clone()).collect()
            }),
            state: input
                .channel("state")
                .and_then(|c| c.fields.first().map(|f| f.name.clone())),
        }
    }
}

impl PeopleSource for RowPeople {
    fn capabilities(&self) -> PeopleCapabilities {
        PeopleCapabilities::default()
    }

    fn rows(&self, term: &str, offset: usize, limit: usize) -> Result<Vec<PersonRow>, SourceError> {
        let term = term.to_lowercase();
        Ok(self
            .rows
            .iter()
            .filter(|r| {
                r.get("label")
                    .unwrap_or(&r.key)
                    .to_lowercase()
                    .contains(&term)
            })
            .skip(offset)
            .take(limit)
            .map(|r| PersonRow::new(&r.key, vec![r.get("label").unwrap_or(&r.key).to_owned()]))
            .collect())
    }

    fn read(&self, key: &str) -> Result<PersonSheet, SourceError> {
        let row = self.rows.iter().find(|r| r.key == key);
        let fields = self
            .fields
            .iter()
            .enumerate()
            .filter_map(|(i, name)| {
                let value = row?.all("columns").get(i).filter(|v| !v.is_empty())?;
                Some(PersonField::new(name, value))
            })
            .chain(
                self.state
                    .as_deref()
                    .zip(row.and_then(|r| r.get("state")).filter(|v| !v.is_empty()))
                    .map(|(name, value)| PersonField::new(name, value)),
            )
            .collect();
        Ok(PersonSheet::new(
            row.and_then(|r| r.get("label")).unwrap_or(key),
            vec![PersonSection::new("", fields)],
        ))
    }
}

#[cfg(test)]
mod tests {
    use fs_components_core::form_read::{ChannelInfo, FieldInfo};
    use fs_components_core::states::Load;

    use super::*;

    fn input() -> FormInput {
        let mut row = FormRow::of("m1", &[("label", "Mara Weidner")]);
        row.values
            .push(("columns".into(), vec!["Moderator".into()]));
        row.values.push(("state".into(), vec!["Owner".into()]));
        let field = |n: &str| FieldInfo {
            name: n.into(),
            block: "field-text".into(),
        };
        FormInput {
            form: "rows@1.0.0".into(),
            rows: vec![row, FormRow::of("m2", &[("label", "Jonas Brink")])],
            channels: vec![
                ChannelInfo {
                    name: "columns".into(),
                    fields: vec![field("role")],
                    choices: Vec::new(),
                },
                ChannelInfo {
                    name: "state".into(),
                    fields: vec![field("level")],
                    choices: Vec::new(),
                },
            ],
            may: Vec::new(),
            load: Load::Ready,
            heads: None,
        }
    }

    #[test]
    fn the_list_is_the_label_and_the_sheet_the_columns_without_the_empty_ones() {
        let people = RowPeople::of(&input());
        let rows = people.rows("mara", 0, 10).expect("rows");
        assert_eq!(rows.len(), 1, "the search finds by the label");
        let sheet = people.read("m1").expect("sheet");
        let text = format!("{sheet:?}");
        assert!(text.contains("Moderator") && text.contains("role"));
        assert!(
            text.contains("Owner") && text.contains("level"),
            "the channel state is the last field"
        );
        let empty = format!("{:?}", people.read("m2").expect("sheet"));
        assert!(!empty.contains("level"), "an empty value is not shown");
    }
}
