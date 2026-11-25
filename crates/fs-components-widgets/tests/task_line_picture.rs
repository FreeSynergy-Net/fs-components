//! `task-row` as a picture: tick, chips, title, code and the description mark are drawn (ink), in the light and the dark scheme.
//! Pictures are kept when `FS_COMP_SHOTS` names a directory.
//!
//! Tests: pixel tests; fixed data, no module.

mod common;

use common::{background, draw, ink, serial, set_dark};
use fs_components_widgets::plan::{Chip, Events, TaskLine, Tone};

fn rows() -> impl blinc_layout::div::ElementBuilder {
    let ev = Events::default();
    let mut a = TaskLine::new("a", "Write the letter");
    a.facts = vec![
        Chip::new("High", Tone::Danger),
        Chip::new("tomorrow", Tone::Neutral),
    ];
    a.described = Some("Has a description".into());
    let mut b = TaskLine::new("b", "Buy bread");
    b.checked = true;
    let mut c = TaskLine::new("c", "Call Anna");
    c.sub = Some("about the trip".into());
    c.code = Some("T-3".into());
    c.facts = vec![Chip::new("Mid", Tone::Warning)];
    let mut col = blinc_layout::div::div().flex_col().w(600.0);
    for r in [a, b, c] {
        col = col.child(r.build("L", &ev));
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
            200,
            if dark {
                "task_line_dark.png"
            } else {
                "task_line_light.png"
            },
        );
        let bg = background(&img);
        assert!(ink(&img, (0, 0, 600, 200), bg) > 600, "dark={dark}");
        assert!(
            ink(&img, (400, 0, 600, 60), bg) > 30,
            "chips at the right, dark={dark}"
        );
    }
    set_dark(false);
}
