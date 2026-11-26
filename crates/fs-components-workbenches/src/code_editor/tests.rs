//! The rules of the code editor over a fake host: typing changes only the draft, save sends exactly one `edit`, discard goes back,
//! a locked text never writes. (The drawing needs a window and is shown by the picture of E5.)
//!
//! Pattern: Fake (a host that keeps the calls) — the rules run over the wiring without a window.

use std::sync::{Arc, Mutex};

use fs_components_core::form_read::{FormInput, FormRow};
use fs_components_core::form_view::{DrawCtx, FormView};
use fs_components_core::form_write::{FormWrite, WriteArg, WriteDone, WriteRefused, WriteValue};
use fs_components_core::states::Load;
use fs_components_styles::StyleId;
use fs_components_words::Locale;

use super::View;
use super::wiring::Wiring;

/// A host that may or may not edit and keeps every call.
type Call = (String, Option<String>, Vec<WriteArg>);

#[derive(Default)]
struct Host {
    allowed: bool,
    /// The host takes the right back (`can` changes while the editor is open).
    revoked: std::sync::atomic::AtomicBool,
    refuse: bool,
    calls: Mutex<Vec<Call>>,
}

impl FormWrite for Host {
    fn can(&self, channel: &str, _record: Option<&str>) -> bool {
        self.allowed
            && !self.revoked.load(std::sync::atomic::Ordering::Relaxed)
            && channel == "edit"
    }

    fn send(
        &self,
        channel: &str,
        record: Option<&str>,
        args: &[WriteArg],
    ) -> Result<WriteDone, WriteRefused> {
        self.calls.lock().unwrap().push((
            channel.to_owned(),
            record.map(str::to_owned),
            args.to_vec(),
        ));
        if self.refuse {
            return Err(WriteRefused::new("conflict"));
        }
        Ok(WriteDone::default())
    }
}

fn input(content: &str, media_type: &str) -> FormInput {
    FormInput {
        form: "single-record@1.1.0".to_owned(),
        rows: vec![FormRow::of(
            "page-1",
            &[
                ("label", "Page"),
                ("content", content),
                ("media-type", media_type),
            ],
        )],
        channels: Vec::new(),
        may: Vec::new(),
        load: Load::Ready,
        heads: None,
    }
}

fn ctx(host: &Arc<Host>, now: &Arc<Mutex<FormInput>>) -> DrawCtx {
    let reread = Arc::clone(now);
    DrawCtx::new(
        fs_components_core::form_view::PlaceInfo::default(),
        (StyleId::Admin, false),
        Arc::new(str::to_owned),
        Locale::english(),
        host.clone(),
        Arc::new(move || reread.lock().unwrap().clone()),
    )
}

fn wiring(host: &Arc<Host>, content: &str, media_type: &str) -> (Wiring, Arc<Mutex<FormInput>>) {
    let now = Arc::new(Mutex::new(input(content, media_type)));
    let w = Wiring::new(input(content, media_type), &ctx(host, &now));
    (w, now)
}

fn host(allowed: bool) -> Arc<Host> {
    Arc::new(Host {
        allowed,
        ..Host::default()
    })
}

fn type_in(w: &Wiring, text: &str) {
    w.draft().editor().set(text);
    w.typed();
}

#[test]
fn the_view_is_the_widget_of_the_block() {
    assert_eq!(View.widget(), "CodeEditor");
}

#[test]
fn typing_changes_only_the_draft() {
    let h = host(true);
    let (w, _) = wiring(&h, "# one", "text/markdown");
    assert!(!w.draft().is_dirty());
    type_in(&w, "# two");
    assert!(w.draft().is_dirty());
    assert_eq!(w.draft().text(), "# two");
    assert!(h.calls.lock().unwrap().is_empty());
}

#[test]
fn save_sends_exactly_one_edit_with_the_whole_text_and_the_draft_follows_the_module() {
    let h = host(true);
    let (w, now) = wiring(&h, "a\nb", "text/markdown");
    type_in(&w, "a\nb\nc");
    // the module will say the new text at the next reading
    *now.lock().unwrap() = input("a\nb\nc", "text/markdown");
    w.save();
    w.save(); // clean now: no second call
    let calls = h.calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "edit");
    assert_eq!(calls[0].1.as_deref(), Some("page-1"));
    assert_eq!(
        calls[0].2,
        vec![WriteArg {
            name: "content".to_owned(),
            value: WriteValue::Text("a\nb\nc".to_owned())
        }]
    );
    assert!(!w.draft().is_dirty());
}

#[test]
fn a_refused_save_keeps_the_draft_and_says_why() {
    let h = Arc::new(Host {
        allowed: true,
        refuse: true,
        ..Host::default()
    });
    let (w, _) = wiring(&h, "x", "text/plain");
    type_in(&w, "y");
    w.save();
    assert!(w.draft().is_dirty());
    assert_eq!(w.refusal().as_deref(), Some("code-editor.refused.conflict"));
    type_in(&w, "yy");
    assert_eq!(w.refusal(), None, "typing clears the old refusal");
}

#[test]
fn discard_goes_back_to_what_the_module_holds_and_writes_nothing() {
    let h = host(true);
    let (w, _) = wiring(&h, "keep", "text/plain");
    type_in(&w, "lost");
    w.discard();
    assert_eq!(w.draft().text(), "keep");
    assert!(!w.draft().is_dirty());
    assert!(h.calls.lock().unwrap().is_empty());
}

#[test]
fn a_locked_text_says_why_and_never_writes() {
    let h = host(false);
    let (w, _) = wiring(&h, "a", "text/plain");
    assert_eq!(w.locked().as_deref(), Some("code-editor.readonly.no-edit"));
    type_in(&w, "b");
    w.save();
    assert!(h.calls.lock().unwrap().is_empty());
    assert_eq!(w.refusal().as_deref(), Some("code-editor.refused.readonly"));
}

#[test]
fn an_editable_text_has_no_reason_and_markdown_has_a_preview() {
    let h = host(true);
    let (md, _) = wiring(&h, "a", "text/markdown");
    let (toml, _) = wiring(&h, "a", "application/toml");
    assert_eq!(md.locked(), None);
    assert!(md.has_preview());
    assert!(!toml.has_preview());
}

#[test]
fn a_new_reading_keeps_what_was_typed_and_a_clean_draft_follows() {
    let h = host(true);
    let (w, _) = wiring(&h, "one", "text/plain");
    w.load(input("two", "text/plain"));
    assert_eq!(w.draft().text(), "two");
    type_in(&w, "mine");
    w.load(input("three", "text/plain"));
    assert_eq!(w.draft().text(), "mine");
    w.discard();
    assert_eq!(w.draft().text(), "three");
}

#[test]
fn the_media_type_picks_the_colours_when_the_feature_is_on() {
    let h = host(true);
    let (w, _) = wiring(&h, "a", "text/markdown");
    assert_eq!(w.draft().media_type(), "text/markdown");
    assert_eq!(
        super::draft::language_of("text/markdown").is_some(),
        cfg!(feature = "syntect")
    );
    assert!(super::draft::language_of("image/png").is_none());
}

#[test]
fn the_language_is_one_message_with_a_parameter() {
    let h = host(true);
    let now = Arc::new(Mutex::new(input("a", "text/markdown")));
    let mut c = ctx(&h, &now);
    c.say = Arc::new(|key: &str| {
        if key == "code-editor.language" {
            "Language: {media-type}".to_owned()
        } else {
            key.to_owned()
        }
    });
    let w = Wiring::new(input("a", "text/markdown"), &c);
    assert_eq!(w.language_label(), "Language: text/markdown");
    // a host without the key: the bare media type, not a sentence put together here
    let (bare, _) = wiring(&h, "a", "application/toml");
    assert_eq!(bare.language_label(), "application/toml");
}

/// Review SOLL 4 (1): a new media type makes a new editor state, so the body must be built again, else the drawn widget would
/// hang on the old state. A new text alone must not (typing would lose its place).
#[test]
fn a_new_media_type_builds_the_body_again_a_new_text_does_not() {
    let h = host(true);
    let (w, _) = wiring(&h, "a", "text/markdown");
    w.mark_built();
    w.load(input("b", "text/markdown"));
    assert!(!w.sync_body(), "only the text changed: the body stays");
    w.load(input("b", "application/toml"));
    // `load` already built it again once; marking it built and asking again finds nothing stale
    w.mark_built();
    assert!(!w.sync_body());
    assert_eq!(w.draft().media_type(), "application/toml");
    assert!(!w.has_preview(), "the preview goes with the media type");
    w.load(input("b", "text/markdown"));
    assert!(w.has_preview());
}

/// Review SOLL 4 (2): `can` changes after a reading: the body is locked or unlocked.
#[test]
fn a_changed_can_builds_the_body_again() {
    let h = host(true);
    let (w, now) = wiring(&h, "a", "text/plain");
    w.mark_built();
    assert_eq!(w.locked(), None);
    h.revoked.store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(w.sync_body(), "locked now: the body is stale");
    assert!(w.locked().is_some());
    w.mark_built();
    assert!(!w.sync_body());
    h.revoked.store(false, std::sync::atomic::Ordering::Relaxed);
    w.load(now.lock().unwrap().clone());
    assert_eq!(w.locked(), None);
    assert!(w.sync_body(), "unlocked again: stale until it is built");
    w.mark_built();
    assert!(!w.sync_body());
}

/// A new media type in a reading builds the body again (once) and changes the language; a new text or typing does not.
#[test]
fn a_new_media_type_in_a_reading_builds_the_body_again_and_changes_the_language() {
    let h = host(true);
    let (w, _) = wiring(&h, "a", "text/markdown");
    w.mark_built();
    type_in(&w, "typed");
    assert_eq!(w.rebuilds(), 0, "typing does not build the body again");
    w.load(input("b", "text/markdown"));
    assert_eq!(w.rebuilds(), 0, "a new text does not either");
    w.load(input("b", "application/toml"));
    assert_eq!(w.rebuilds(), 1, "a new media type does");
    assert_eq!(w.draft().media_type(), "application/toml");
    assert!(!w.has_preview());
    w.mark_built();
    w.load(input("c", "application/toml"));
    assert_eq!(w.rebuilds(), 1, "built for this shape: nothing stale");
}
