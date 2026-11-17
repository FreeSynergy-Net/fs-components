//! P1 probe: K cases of the grey ghost text (typing over it, taking it with a key, copy without it).
//!
//! Tests: run by `festzeichen.rs` against the real field and against each mutant.

use super::address::Mutation;
use super::typist::Typist;
use super::{ok, parts};

/// K12: the typed `@` overwrites the grey one: the sign turns solid, the `@` is not in the text, and there is no second `@`.
pub fn k12_typing_overwrites_ghost(m: Mutation) -> Result<(), String> {
    let mut t = Typist::new(m);
    t.text("ada");
    ok(
        !t.field.real(0) && t.field.address() == "ada",
        format!("ghost taken too early: {:?}", t.field.address()),
    )?;
    t.text("@");
    ok(
        t.field.real(0) && t.field.address() == "ada@" && t.field.focus() == Some(1),
        format!("{:?} {}", t.field.address(), parts(&t.field)),
    )
}

/// K13: the accept key (Enter or the right arrow) takes the grey `@` as it is and the writing goes on in the server; nothing is doubled.
pub fn k13_accept_ghost_sign(m: Mutation) -> Result<(), String> {
    for key in [13, 39] {
        let mut t = Typist::with(m);
        t.text("ada");
        t.press(key, false);
        let f = &t.field;
        ok(
            f.real(0) && f.address() == "ada@" && f.focus() == Some(1) && f.cursor(1) == 0,
            format!("key {key}: {:?} {}", f.address(), parts(f)),
        )?;
        t.text("x");
        ok(
            t.field.address() == "ada@x",
            format!("key {key} then x: {:?}", t.field.address()),
        )?;
    }
    Ok(())
}

/// K14: the suggested ending is taken once: `o` + grey `rg` becomes `org`.
pub fn k14_accept_ending(m: Mutation) -> Result<(), String> {
    let mut t = Typist::new(m);
    t.text("a@b.o");
    ok(
        t.field.ghost_tld() == "rg",
        format!("ghost {:?}", t.field.ghost_tld()),
    )?;
    t.press(13, false);
    ok(
        t.field.value(2) == "org" && t.field.address() == "a@b.org",
        format!("{:?}", t.field.address()),
    )
}

/// K15: what is copied (and what fs-types sees) is the real text only: no grey sign, no grey ending.
pub fn k15_copy_without_ghost(m: Mutation) -> Result<(), String> {
    let mut t = Typist::new(m);
    t.text("ada");
    ok(
        t.field.address() == "ada",
        format!("{:?}", t.field.address()),
    )?;
    t.text("@b.o");
    t.field.go(2, None);
    t.field.key_up(67, true);
    let got = t.field.st.lock().unwrap().clip.last().cloned();
    ok(
        got.as_deref() == Some("ada@b.o"),
        format!("copy gave {got:?}"),
    )
}

/// K16: End takes the suggested ending from anywhere; Enter with nothing grey does nothing (the form may submit); Tab never takes anything.
pub fn k16_end_enter_tab(m: Mutation) -> Result<(), String> {
    let mut t = Typist::new(m);
    t.text("a@b.o");
    t.field.go(1, None);
    t.press(35, false);
    ok(
        t.field.value(2) == "org",
        format!("End: {}", parts(&t.field)),
    )?;
    t.press(13, false); // nothing grey left: Enter changes nothing
    ok(
        t.field.value(2) == "org" && t.field.address() == "a@b.org",
        format!("Enter without grey: {}", parts(&t.field)),
    )?;
    let mut u = Typist::new(m);
    u.text("a@b.o");
    u.press(9, false); // Tab: no suggestion taken
    ok(
        u.field.value(2) == "o",
        format!("Tab took it: {}", parts(&u.field)),
    )
}
