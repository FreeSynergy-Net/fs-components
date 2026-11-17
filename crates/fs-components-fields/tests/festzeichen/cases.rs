//! P1 probe: K cases of the fixed signs (jumps, backspace, arrows, paste, copy, validity).
//!
//! Tests: run by `festzeichen.rs` against the real field and against each mutant.

use super::address::{AddressField, Mutation};
use super::split::Change;
use super::typist::Typist;
use super::{TLDS, ok, parts};
use blinc_layout::widgets::text_input::InputType;

/// K1: `@` typed in the local part jumps to the server, and no `@` stays in any part.
pub fn k1_at_jumps(m: Mutation) -> Result<(), String> {
    let mut t = Typist::new(m);
    t.text("ada@");
    let f = &t.field;
    ok(
        f.focus() == Some(1) && f.value(0) == "ada",
        format!("no jump {}", parts(f)),
    )?;
    t.text("x");
    ok(
        t.field.value(1) == "x" && (0..3).all(|i| !t.field.value(i).contains('@')),
        parts(&t.field),
    )
}

/// K2: valid addresses typed key by key are accepted by fs-types, invalid ones are not (the sign `.` is in the field, `ada@localhost` is impossible).
pub fn k2_typed_addresses(m: Mutation) -> Result<(), String> {
    for good in [
        "ada@example.org",
        "a.b+tag@mail.example.co.uk",
        "x_y@a-b.de",
        "o'neil@example.ie",
        "Ada@Example.ORG",
    ] {
        let mut t = Typist::new(m);
        t.text(good);
        ok(
            t.field.is_valid(),
            format!(
                "{good} rejected: {} -> {:?}",
                parts(&t.field),
                t.field.address()
            ),
        )?;
        ok(
            t.field.address().to_lowercase() == good.to_lowercase(),
            format!("{good} became {}", t.field.address()),
        )?;
    }
    for bad in ["ada@localhost", "ada", "ada@", "ada@example."] {
        let mut t = Typist::new(m);
        t.text(bad);
        ok(
            !t.field.is_valid(),
            format!("{bad} accepted as {:?}", t.field.address()),
        )?;
    }
    Ok(())
}

/// K3 (E3): a `.` in a filled tld moves the tld to the server and leaves the tld empty; mail.example has two dots.
pub fn k3_server_with_dots(m: Mutation) -> Result<(), String> {
    let mut t = Typist::new(m);
    t.text("a@mail.example.");
    let f = &t.field;
    ok(
        f.value(1) == "mail.example" && f.value(2).is_empty() && f.focus() == Some(2),
        parts(f),
    )
}

/// K4 (E16): a `.` in the middle of the server stays a free dot; no jump, the rest stays.
pub fn k4_mid_dot(m: Mutation) -> Result<(), String> {
    let mut t = Typist::new(m);
    t.text("a@mailbeispiel");
    t.keys(37, 8);
    t.text(".");
    let f = &t.field;
    ok(
        f.value(1) == "mail.beispiel"
            && f.focus() == Some(1)
            && f.cursor(1) == 5
            && f.value(2).is_empty(),
        parts(f),
    )
}

/// K5 (E4): Backspace from the end of the tld to the start of the local part removes every typed character and never a fixed sign.
pub fn k5_backspace(m: Mutation) -> Result<(), String> {
    let mut t = Typist::new(m);
    t.text("ada@x.org");
    ok(t.field.value(2) == "org", parts(&t.field))?;
    t.keys(8, 3);
    ok(
        t.field.value(2).is_empty() && t.field.focus() == Some(2),
        parts(&t.field),
    )?;
    t.keys(8, 1); // empty tld: back into the server, nothing deleted
    ok(
        t.field.focus() == Some(1) && t.field.value(1) == "x",
        format!("no way back {}", parts(&t.field)),
    )?;
    t.keys(8, 1);
    t.keys(8, 1); // empty server: back into the local part
    ok(
        t.field.focus() == Some(0) && t.field.value(0) == "ada",
        parts(&t.field),
    )?;
    t.keys(8, 3);
    ok((0..3).all(|i| t.field.value(i).is_empty()), parts(&t.field))
}

/// K6: Backspace at the START of a filled part goes over the fixed sign and deletes nothing.
pub fn k6_backspace_at_start(m: Mutation) -> Result<(), String> {
    let mut t = Typist::new(m);
    t.text("ada@x.org");
    t.field.go(1, Some(0));
    t.keys(8, 1);
    ok(
        t.field.focus() == Some(0) && t.field.value(0) == "ada" && t.field.cursor(0) == 3,
        parts(&t.field),
    )
}

/// K7: arrow keys move through the parts as one line; at the edge they cross the fixed sign.
pub fn k7_arrows(m: Mutation) -> Result<(), String> {
    let mut t = Typist::new(m);
    t.text("ada@mail.example.org");
    t.field.go(2, Some(0));
    t.keys(37, 1);
    ok(
        t.field.focus() == Some(1) && t.field.cursor(1) == 12,
        format!("left over the dot {}", parts(&t.field)),
    )?;
    t.keys(39, 1);
    ok(
        t.field.focus() == Some(2) && t.field.cursor(2) == 0,
        format!("right over the dot {}", parts(&t.field)),
    )?;
    t.field.go(1, Some(1));
    t.keys(37, 1);
    ok(
        t.field.focus() == Some(1) && t.field.cursor(1) == 0,
        format!("inside a part {}", parts(&t.field)),
    )
}

/// K8 (E2): pasting a whole address spreads it; the cover is removed and shown; server mail.example, tld org; two `@` keep everything in local.
pub fn k8_paste(m: Mutation) -> Result<(), String> {
    for (text, local, server, tld) in [
        ("a@mail.example.org", "a", "mail.example", "org"),
        ("Ada <ada@mail.example.org>", "ada", "mail.example", "org"),
        ("mailto:ada@x.org", "ada", "x", "org"),
        ("  ada@x.org ", "ada", "x", "org"),
    ] {
        let t = Typist::new(m);
        t.field.paste(0, text);
        let f = &t.field;
        ok(
            (f.value(0), f.value(1), f.value(2)) == (local.into(), server.into(), tld.into()),
            format!("{text:?} -> {}", parts(f)),
        )?;
    }
    let t = Typist::new(m);
    t.field.paste(0, "Ada <ada@x.org>");
    ok(
        t.field.st.lock().unwrap().shown.contains(&Change::Cover),
        "cover removed silently",
    )?;
    t.field.paste(0, "a@b@c.org");
    ok(
        t.field.value(0) == "a@b@c.org"
            && t.field.st.lock().unwrap().shown.contains(&Change::ManyAt),
        parts(&t.field),
    )
}

/// K9 (E5): Ctrl+A marks all three parts and Ctrl+C gives the whole address with both fixed signs.
pub fn k9_copy(m: Mutation) -> Result<(), String> {
    let mut t = Typist::new(m);
    t.text("ada@mail.example.org");
    t.field.go(0, None);
    t.press(65, true); // Ctrl+A goes through the tree: it does not touch the clipboard
    ok(
        t.field
            .segs
            .iter()
            .all(|s| s.lock().unwrap().selection_start == Some(0)),
        "not all marked",
    )?;
    t.field.key_up(67, true); // Ctrl+C: called directly, Blinc would write the desktop clipboard
    let got = t.field.st.lock().unwrap().clip.last().cloned();
    ok(
        got.as_deref() == Some("ada@mail.example.org"),
        format!("copy gave {got:?}"),
    )
}

/// K10: no part uses Blinc's own e-mail check (a third source of the rule): all are `InputType::Text`.
pub fn k10_only_text(m: Mutation) -> Result<(), String> {
    let f = AddressField::new(m, TLDS).unwrap();
    ok(
        f.segs
            .iter()
            .all(|s| s.lock().unwrap().input_type == InputType::Text),
        "a part is not InputType::Text",
    )
}

/// K11: `@` in server or tld is not taken and says why (nothing silent).
pub fn k11_second_at(m: Mutation) -> Result<(), String> {
    let mut t = Typist::new(m);
    t.text("ada@x@");
    let f = &t.field;
    ok(
        f.value(1) == "x" && f.st.lock().unwrap().hint == Some("problem.email.many-at"),
        parts(f),
    )
}
