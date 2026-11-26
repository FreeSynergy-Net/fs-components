//! Property of `rows_form`: the cell under a head belongs to the field of that head, whatever the other fields carry (none, one or
//! several values, in any interleaving on the wire).
//!
//! Pattern: Property test (proptest) over the Adapter.

use fs_components_core::form_read::{ChannelInfo, FieldInfo, FormInput, FormRow};
use fs_components_core::states::Load;
use proptest::prelude::*;

use crate::rows_form::columns_and_cells;
use crate::rows_form_tests::ctx;

const FIELDS: [&str; 4] = ["emails", "phones", "organizations", "titles"];

proptest! {
    #[test]
    fn the_cell_under_a_head_belongs_to_its_field(wire in prop::collection::vec(0usize..4, 0..12)) {
        // `wire`: the field of each value as the module sends them (any interleaving); the text names field and rank
        let mut seen = [0usize; 4];
        let mut pairs: Vec<(&str, String)> = Vec::new();
        let mut want: Vec<Vec<String>> = vec![Vec::new(); 4];
        for &i in &wire {
            let text = format!("{i}-{}", seen[i]);
            seen[i] += 1;
            want[i].push(text.clone());
            pairs.push((FIELDS[i], text));
        }
        let map = vec![("columns".to_owned(), FIELDS.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>())];
        let row = FormRow::from_values("k", Vec::new(), &map, pairs.iter().map(|(p, t)| (*p, t.as_str())));
        let input = FormInput {
            form: "rows@1.0.0".to_owned(),
            rows: vec![row],
            channels: vec![ChannelInfo {
                name: "columns".to_owned(),
                fields: FIELDS.iter().map(|n| FieldInfo { name: (*n).to_owned(), block: "field-text".to_owned(), label: String::new() }).collect(),
                choices: Vec::new(),
            }],
            may: Vec::new(),
            load: Load::Ready,
            heads: None,
        };
        let (cols, cells) = columns_and_cells(&input, &ctx());
        prop_assert_eq!(cols.len(), 1 + FIELDS.len());
        prop_assert_eq!(cells[0].1.len(), cols.len());
        for (i, f) in FIELDS.iter().enumerate() {
            prop_assert_eq!(&cols[1 + i].key, f);
            prop_assert_eq!(&cells[0].1[1 + i], &want[i].join("\n"));
        }
    }
}
