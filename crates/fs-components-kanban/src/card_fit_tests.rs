//! Tests of the room of the line of a card (`card_fit`) and of the marks of the assignees (`meta`).
//!
//! Pattern: none; tests.
#![allow(clippy::pedantic)]

use crate::Card;
use crate::card_fit::{fit, share, width_of};
use crate::meta::assignee_marks;

#[test]
fn short_pieces_keep_their_width_and_the_long_ones_share_the_rest() {
    let given = share(&[20.0, 300.0, 300.0], 220.0);
    assert!(
        (given[0] - 20.0).abs() < 0.01,
        "the short one keeps its width"
    );
    assert!((given[1] - 100.0).abs() < 0.01 && (given[2] - 100.0).abs() < 0.01);
    assert!(
        given.iter().sum::<f32>() <= 220.01,
        "never more than the room"
    );
}

#[test]
fn nothing_is_taken_when_everything_fits() {
    assert_eq!(share(&[10.0, 20.0], 100.0), vec![10.0, 20.0]);
}

#[test]
fn no_room_gives_no_room_and_never_panics() {
    assert_eq!(share(&[10.0, 20.0], -5.0), vec![0.0, 0.0]);
    assert!(share(&[], 10.0).is_empty());
}

#[test]
fn a_text_that_does_not_fit_ends_in_an_ellipsis_never_in_a_cut_letter() {
    let whole = "ada, bob, carol, dave, erin";
    let room = width_of(whole) / 2.0;
    let cut = fit(whole, room);
    assert!(cut.ends_with('\u{2026}'), "{cut}");
    assert!(cut.len() < whole.len());
    assert!(width_of(&cut) <= room + 0.5, "the cut text fits its room");
    assert_eq!(fit("ada", 500.0), "ada", "a text that fits is untouched");
}

#[test]
fn assignees_carry_their_whole_names_and_the_rest_is_counted() {
    let mut card = Card::new("a", "x");
    card.assignees = ["Ada Lovelace", "Bob", "Carol King", "Dave Grohl", "Erin"]
        .map(String::from)
        .to_vec();
    let marks = assignee_marks(&card);
    assert_eq!(
        marks,
        vec![
            ("AL".into(), "Ada Lovelace".into()),
            ("B".into(), "Bob".into()),
            ("CK".into(), "Carol King".into()),
            ("+2".into(), "Dave Grohl, Erin".into()),
        ]
    );
}
