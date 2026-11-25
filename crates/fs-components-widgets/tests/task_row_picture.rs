//! `task-row` as a picture: tick, priority chip, title, date and the description mark are drawn (ink), in the light and the dark scheme.
//! Pictures are kept when `FS_COMP_SHOTS` names a directory.
//!
//! Tests: pixel tests; fixed data, no module.

mod common;

use common::{background, draw, ink, serial, set_dark};
use fs_components_widgets::task_row::{TaskRow, Tone};

fn rows() -> impl blinc_layout::div::ElementBuilder {
    let mut col = blinc_layout::div::div().flex_col().w(600.0);
    for r in [
        TaskRow::new("a", "Write the letter")
            .prio("High", Tone::Danger)
            .due("tomorrow")
            .described("Has a description"),
        TaskRow::new("b", "Buy bread")
            .done()
            .prio("Low", Tone::Plain),
        TaskRow::new("c", "Call Anna")
            .subline("about the trip")
            .prio("Mid", Tone::Warning)
            .selected(),
    ] {
        col = col.child(r.build());
    }
    col
}

#[test]
fn the_rows_draw_in_both_schemes() {
    let _g = serial();
    for dark in [false, true] {
        set_dark(dark);
        let img = draw(
            rows(),
            600,
            140,
            if dark {
                "task_row_dark.png"
            } else {
                "task_row_light.png"
            },
        );
        let bg = background(&img);
        assert!(ink(&img, (0, 0, 600, 140), bg) > 600, "dark={dark}");
        assert!(
            ink(&img, (440, 0, 600, 40), bg) > 30,
            "chip, date and mark at the right, dark={dark}"
        );
    }
    set_dark(false);
}
