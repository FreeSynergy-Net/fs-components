//! The agenda: the items of the next days as a list by day, days without items left out.
//!
//! Pattern: View function — the days and items come from the state and the source; the title of each day is the CLDR name of
//! `DrawCtx.dates` (`Dienstag, 10. November`), the words ("all day", "nothing planned") are the catalog's. A click on a day
//! selects it, a click on a row selects the item.

use blinc_layout::div::{Div, div};
use blinc_layout::widgets::scroll::scroll;
use fs_components_fields::Look;
use fs_components_words::Width;

use super::parts::{item_look, stripe};
use crate::Day;
use crate::handle::CalendarGrid;
use crate::model::{GridItem, Slot};
use crate::names::day_title;

/// The agenda filling `width` x `height`.
pub fn agenda(grid: &CalendarGrid, items: &[GridItem], width: f32, height: f32) -> Div {
    let (look, s, pal) = (grid.look, grid.look.style, grid.look.palette());
    let state = grid.snapshot();
    let mut list = div()
        .flex_col()
        .w(width)
        .gap_px(s.spacing.xs)
        .id("cal-agenda");
    let mut any = false;
    for day in state.days() {
        let mut of_day: Vec<&GridItem> = items
            .iter()
            .filter(|it| it.start.day <= day && day <= it.last_day())
            .collect();
        if of_day.is_empty() {
            continue;
        }
        any = true;
        of_day.sort_by_key(|it| (!it.all_day, it.start.minute));
        let today = state.today == Some(day);
        let (g, slot) = (grid.clone(), Slot { day, minute: None });
        list = list.child(
            div()
                .id(format!("cal-agenda-day-{}-{}", day.month(), day.day()))
                .padding_x_px(s.spacing.xs)
                .border_bottom(1.0, Look::color(pal.border))
                .on_click(move |_| g.select_slot(slot))
                .child(look.text(
                    day_title(&*grid.dates, day, Width::Long),
                    s.text.body,
                    if today { pal.accent_text } else { pal.text },
                )),
        );
        for it in of_day {
            list = list.child(row(
                grid,
                day,
                it,
                state.item.as_deref() == Some(it.key.as_str()),
            ));
        }
    }
    if !any {
        list = list.child(look.text(grid.say.say("calendar-empty"), s.text.body, pal.text_weak));
    }
    div()
        .w(width)
        .h(height)
        .child(scroll().w(width).h(height).content(list))
}

/// The time of `item` on `day`: the word "all day", or the clock times, or nothing on a later day of a long item.
fn time_of(grid: &CalendarGrid, day: Day, it: &GridItem) -> String {
    let clock = |m: u16| format!("{:02}:{:02}", m / 60, m % 60);
    if it.all_day {
        grid.say.say("calendar-all-day")
    } else if it.start.day != day {
        String::new()
    } else if it.end.day == day {
        format!("{}\u{2013}{}", clock(it.start.minute), clock(it.end.minute))
    } else {
        clock(it.start.minute)
    }
}

fn row(grid: &CalendarGrid, day: Day, it: &GridItem, selected: bool) -> Div {
    let (look, s, pal) = (grid.look, grid.look.style, grid.look.palette());
    let (g, key) = (grid.clone(), it.key.clone());
    let color = item_look(&look, it);
    let mut line = div()
        .id(format!(
            "cal-agenda-item-{}-{}-{}",
            it.key,
            day.month(),
            day.day()
        ))
        .flex_row()
        .items_center()
        .gap_px(s.spacing.s)
        .padding_x_px(s.spacing.s)
        .rounded(s.radii.control.min(4.0))
        .on_click(move |_| g.select_item(&key))
        .child({
            let mut mark = stripe(&color, 4.0, s.text.caption * 1.4).rounded(2.0);
            if let Some(o) = color.outline {
                mark = mark.border(1.0, Look::color(o));
            }
            mark.id(format!(
                "cal-agenda-mark-{}-{}-{}",
                it.key,
                day.month(),
                day.day()
            ))
        })
        .child(div().w(s.text.caption * 9.0).child(look.text(
            time_of(grid, day, it),
            s.text.caption,
            pal.text_weak,
        )))
        .child(look.text(&it.title, s.text.body, pal.text));
    if selected {
        line = line.border(2.0, Look::color(pal.accent_text));
    }
    line
}
